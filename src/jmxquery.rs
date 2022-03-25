use crate::{
    payload::{MBeansAttributeInfo, QueryMBeansTree},
    testconfig::{JmxMetric, SubSystem, TestConfig, ThingworxServer},
    twxquery::construct_headers,
};
use chrono::offset::Utc;
use chrono::DateTime;
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{header::HeaderMap, Client};

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

pub type JmxObjectNameList = Vec<(
    String,         // Measurement Name eventually, like: jmx_c3p0_connections, jmx_memory_status
    Vec<String>, // list of objectNames, like: ["java.lang:type=MemoryPool,name=G1 Survivor Space","java.lang:type=MemoryPool,name=G1 Old Gen"]
    Option<String>, // optional tag to grab name label, default is "name". all value has to be string
    Vec<String>,    // list of query result to filter. default is empty, which means no filter
)>;
pub async fn refresh_jmx(
    tc: TestConfig,
    mut writer: evmap::WriteHandle<String, JmxObjectNameList>,
    sleeping: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    loop {
        for server in tc.thingworx_servers.iter() {
            if let Some(ref jmx_configs) = server.jmx_metrics {
                let headers = construct_headers(&server.app_key);
                let url = server.get_query_mbeanstree_url();
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(20))
                    .danger_accept_invalid_certs(true)
                    .build()?;
                log::debug!("JMX MBeans query url:{},jmx_configs:{:?}", url, jmx_configs);
                let res = match client.post(url).headers(headers.clone()).send().await {
                    Ok(res) => res,
                    Err(e) => {
                        log::error!("JMX MBeans query service error:{:?}", e);
                        continue;
                    }
                };

                if !res.status().is_success() {
                    log::error!("JMX MBeans query :{} failed", server.name);
                    continue;
                }

                let mbeans: QueryMBeansTree = match res.json::<QueryMBeansTree>().await {
                    Ok(mbeans) => mbeans,
                    Err(e) => {
                        log::error!(
                            "JMX MBeans query :{} failed to parse result:{:?}",
                            server.name,
                            e
                        );
                        continue;
                    }
                };

                let mut jmx_metrics_vec = Vec::new();
                for JmxMetric {
                    name,
                    object_name_pattern,
                    name_label_alternative,
                    metrics,
                } in jmx_configs.iter()
                {
                    let mut object_names = vec![];
                    for row in mbeans.rows.iter() {
                        if row.object_name.starts_with(object_name_pattern) {
                            object_names.push(row.object_name.clone());
                        }
                    }
                    log::info!(
                        "JMX MBeans query :{} success, got {} JMX MBean(s) for {},metrics:{:?}",
                        server.name,
                        object_names.len(),
                        name,
                        metrics
                    );
                    jmx_metrics_vec.push((
                        name.to_owned(),
                        object_names,
                        name_label_alternative.clone(),
                        metrics.clone(),
                    ));
                }

                writer.update(server.name.clone(), jmx_metrics_vec);
            }
        }
        writer.refresh();
        sleeping.store(true, Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_secs(tc.refresh_server_interval)).await;
        sleeping.store(false, Ordering::SeqCst);
    }
    // Ok(())
}

pub async fn repeated_jmx_query(
    server: &ThingworxServer,
    measurement: String,
    object_name_list: Vec<String>,
    name_alternative: Option<String>,
    metrics: Vec<String>,
    sender: tokio::sync::mpsc::Sender<Vec<WriteQuery>>,
    query_timeout: u64,
) -> anyhow::Result<()> {
    let url = server.get_mbean_attributeinfo_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(query_timeout))
        .danger_accept_invalid_certs(true)
        .build()?;
    log::debug!("JMX MBeans query url:{},metrics:{}", url, metrics.join(","));
    let headers = construct_headers(&server.app_key);
    for object_name in object_name_list.iter() {
        let jmx_subsystem = SubSystem {
            name: measurement.clone(),
            options: Some(metrics.clone()),
            enabled: true,
            sanitize: false,
            split_desc_asprefix: false,
        };

        let payload = serde_json::json!({
            "notWritableOnly": true,
            "showPreview": true,
            "mbeanName": object_name.clone(),
        });

        let mut additional_tags = HashMap::new();
        lazy_static! {
            static ref RE: Regex = Regex::new(r"name=(.*?)(&|$|,)").unwrap();
        }
        let sub_name = match RE.captures(object_name.as_str()) {
            None => "Default".to_owned(),
            Some(caps) => caps[1].to_string(),
        };
        additional_tags.insert("sub_name".to_string(), sub_name);

        let result = match query_jmx_metrics(
            client.clone(),
            &url,
            &headers,
            payload.to_string(),
            &jmx_subsystem,
            &server.name,
            Some(additional_tags),
            name_alternative.clone(),
        )
        .await
        {
            Ok(result) => result,
            Err(e) => {
                log::error!("JMX MBeans query error:{:?}", e);
                return Ok(());
            }
        };
        log::debug!(
            "JMX MBeans query:{} metrics result:{}",
            object_name,
            result.len()
        );
        let _ = sender.send(result).await;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn query_jmx_metrics(
    client: Client,
    url: &str,
    headers: &HeaderMap,
    payload: String,
    subsystem: &SubSystem,
    platform: &str,
    additional_tags: Option<HashMap<String, String>>,
    name_alternative: Option<String>,
) -> anyhow::Result<Vec<WriteQuery>> {
    let mut result = vec![];
    let response_start = SystemTime::now();

    // if this step is error, likely the server is not responsive.
    // so we should not continue to query the metrics.
    // for the rest of the metrics, we will just handle the error within this block.
    let payload_backup = payload.clone();
    let res = client
        .post(url)
        .headers(headers.clone())
        .body(payload)
        .send()
        .await?;
    if !res.status().is_success() {
        // return Err(anyhow::anyhow!("Subsystem metrics query:{} failed", url));
        log::error!(
            "Subsystem metrics query:{} failed,payload:{}",
            url,
            payload_backup
        );
        return Ok(result); // return empty result
    }

    let mbeanattinfo: MBeansAttributeInfo = match res.json::<MBeansAttributeInfo>().await {
        Ok(mbeanattinfo) => mbeanattinfo,
        Err(e) => {
            log::error!(
                "Subsystem metrics query:{} failed to parse result:{:?}, payload:{}",
                url,
                e,
                payload_backup
            );
            return Ok(result); // return empty result
        }
    };

    let response_time = match response_start.elapsed() {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };

    let system_time = SystemTime::now();
    let timestamp: DateTime<Utc> = system_time.into();
    let mut query = Timestamp::Milliseconds(timestamp.timestamp_millis().try_into().unwrap())
        .into_query(&subsystem.name)
        // .add_tag("Provider", provider.to_string())
        .add_tag("Platform", platform.to_string());

    // we can consume all rows here.
    log::debug!(
        "JMX query for {} has results:{}",
        subsystem.name,
        mbeanattinfo.rows.len()
    );

    let mut new_sub_name = "Default".to_owned();
    for row in mbeanattinfo.rows {
        // only process metrics if the subsystem has limited options for metrics names.
        if let Some(ref options) = subsystem.options {
            if !options.is_empty() {
                log::trace!(
                    "options:{:?}, row.name:{},row_type:{}",
                    options,
                    row.name,
                    row.type_
                );
            }
            if !options.is_empty() && !options.contains(&row.name) {
                continue;
            }
        }
        if row.preview.is_empty() {
            continue;
        }
        log::trace!("JMX metrics query, row:{:?}", row);
        if let Some(ref name_alternative) = name_alternative {
            if name_alternative == &row.name {
                new_sub_name = format!("sub_{}", row.preview);
                new_sub_name = new_sub_name
                    .replace(' ', "_")
                    .replace('.', "_")
                    .replace(',', "_")
                    .replace('[', "_")
                    .replace(']', "_");
            }
        }
        if row.type_ == "boolean" {
            match row.preview.parse::<bool>() {
                Err(_) => continue,
                Ok(value) => {
                    query = query.add_field(row.name, value);
                }
            }
        } else if row.type_ == "int" || row.type_ == "java.lang.Integer" {
            match row.preview.parse::<i32>() {
                Err(_) => continue,
                Ok(value) => {
                    query = query.add_field(row.name, value);
                }
            }
        } else if row.type_ == "long" || row.type_ == "java.lang.Long" {
            match row.preview.parse::<i64>() {
                Err(_) => continue,
                Ok(value) => {
                    query = query.add_field(row.name, value);
                }
            }
        } else if row.type_ == "float" || row.type_ == "java.lang.Float" {
            match row.preview.parse::<f32>() {
                Err(_) => continue,
                Ok(value) => {
                    query = query.add_field(row.name, value);
                }
            }
        } else {
            let value = row.preview.to_string();
            query = query.add_field(row.name, value);
        }
    }

    if let Some(ref additional_tags) = additional_tags {
        // Metrics from all JMX metrics will be in a dummy subsystem.
        // Therefore, it requires an additional tag to be added, which is the connection server name.
        let mut need_replace_subname = name_alternative.is_some();
        for (key, value) in additional_tags {
            // this can be optimized in future to avoid copy.
            // it should directly consume the hashmap.

            if need_replace_subname && key == "sub_name" {
                need_replace_subname = false;
                query = query.add_tag(key.clone(), new_sub_name.clone());
                continue;
            }
            query = query.add_tag(key.clone(), value.clone());
        }
    }

    query = query.add_field("ResponseTime", response_time as i64);
    result.push(query);

    Ok(result)
}
