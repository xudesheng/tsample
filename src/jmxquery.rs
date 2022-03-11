use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::SystemTime,
};

use crate::{
    payload::{MBeansAttributeInfo, QueryMBeansTree},
    testconfig::{SubSystem, TestConfig, ThingworxServer},
    twxquery::construct_headers,
};
use chrono::offset::Utc;
use chrono::DateTime;
use influxdb::{InfluxDbWriteable, Timestamp, WriteQuery};
use reqwest::{header::HeaderMap, Client};
use serde_json::Value as JsonValue;

pub async fn refresh_c3p0(
    tc: TestConfig,
    mut writer: evmap::WriteHandle<String, (Vec<String>, Vec<String>)>,
    sleeping: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    loop {
        for server in tc.thingworx_servers.iter() {
            if let Some(ref c3p0_config) = server.c3p0_metrics {
                // if it has a name list configured, then we ignore it.
                if !c3p0_config.names.is_empty() {
                    continue;
                }
                let headers = construct_headers(&server.app_key);
                let url = server.get_query_mbeanstree_url();
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(20))
                    .danger_accept_invalid_certs(true)
                    .build()?;
                log::debug!("JMX MBeans query url:{}", url);
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

                let mut names = vec![];
                for row in mbeans.rows {
                    if row
                        .object_name
                        .starts_with("com.mchange.v2.c3p0:type=PooledDataSource,")
                    {
                        names.push(row.object_name.clone());
                    }
                }
                log::info!(
                    "JMX MBeans query :{} success, got {} C3P0 MBean(s)",
                    server.name,
                    names.len()
                );
                writer.update(server.name.clone(), (names, c3p0_config.metrics.clone()));
            }
        }
        writer.refresh();
        sleeping.store(true, Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_secs(tc.refresh_server_interval)).await;
        sleeping.store(false, Ordering::SeqCst);
    }
    // Ok(())
}

pub async fn repeated_c3p0_query(
    server: &ThingworxServer,
    c3p0_name: &str,
    metrics: Vec<String>,
    sender: tokio::sync::mpsc::Sender<Vec<WriteQuery>>,
    query_timeout: u64,
) -> anyhow::Result<()> {
    let url = server.get_mbean_attributeinfo_url();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(query_timeout))
        .danger_accept_invalid_certs(true)
        .build()?;
    log::debug!("JMX MBeans query url:{}", url);
    let headers = construct_headers(&server.app_key);
    let c3p0_subsystem = SubSystem {
        name: "C3P0PooledDataSource".to_string(),
        options: Some(metrics),
        enabled: true,
        sanitize: false,
        split_desc_asprefix: false,
    };
    let mut payload = HashMap::new();
    payload.insert("notWritableOnly", JsonValue::Bool(true));
    payload.insert("showPreview", JsonValue::Bool(true));
    payload.insert("mbeanName", JsonValue::String(c3p0_name.to_string()));

    let payload = serde_json::json!({
        "notWritableOnly": true,
        "showPreview": true,
        "mbeanName": c3p0_name,
    });

    let mut additional_tags = HashMap::new();
    additional_tags.insert("c3p0name".to_string(), c3p0_name.to_string());
    let result = match query_c3p0_metrics(
        client,
        &url,
        &headers,
        payload.to_string(),
        &c3p0_subsystem,
        &server.name,
        Some(additional_tags),
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
        c3p0_name,
        result.len()
    );
    let _ = sender.send(result).await;
    Ok(())
}

async fn query_c3p0_metrics(
    client: Client,
    url: &str,
    headers: &HeaderMap,
    payload: String,
    subsystem: &SubSystem,
    platform: &str,
    additional_tags: Option<HashMap<String, String>>,
) -> anyhow::Result<Vec<WriteQuery>> {
    let mut result = vec![];
    let response_start = SystemTime::now();

    // if this step is error, likely the server is not responsive.
    // so we should not continue to query the metrics.
    // for the rest of the metrics, we will just handle the error within this block.
    let res = client
        .post(url)
        .headers(headers.clone())
        .body(payload)
        .send()
        .await?;
    if !res.status().is_success() {
        // return Err(anyhow::anyhow!("Subsystem metrics query:{} failed", url));
        log::error!("Subsystem metrics query:{} failed", url);
        return Ok(result); // return empty result
    }

    let mbeanattinfo: MBeansAttributeInfo = match res.json::<MBeansAttributeInfo>().await {
        Ok(mbeanattinfo) => mbeanattinfo,
        Err(e) => {
            log::error!(
                "Subsystem metrics query:{} failed to parse result:{:?}",
                url,
                e
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
    if let Some(ref additional_tags) = additional_tags {
        // Metrics from all connection servers will be in a dummy subsystem 'ConnectionServer'.
        // Therefore, it requires an additional tag to be added, which is the connection server name.
        for (key, value) in additional_tags {
            // this can be optimized in future to avoid copy.
            // it should directly consume the hashmap.
            query = query.add_tag(key.clone(), value.clone());
        }
    }

    // we can consume all rows here.
    log::debug!(
        "JMX query for {} has results:{}",
        subsystem.name,
        mbeanattinfo.rows.len()
    );
    for row in mbeanattinfo.rows {
        // only process metrics if the subsystem has limited options for metrics names.
        if let Some(ref options) = subsystem.options {
            if !options.is_empty() && !options.contains(&row.name) {
                continue;
            }
        }
        if row.preview.is_empty() {
            continue;
        }
        log::trace!("JMX metrics query, row:{:?}", row);
        if row.type_ == "int" {
            let value = row.preview.parse::<i32>().unwrap_or(0);
            query = query.add_field(row.name, value);
        } else if row.type_ == "long" {
            let value = row.preview.parse::<i64>().unwrap_or(0);
            query = query.add_field(row.name, value);
        } else if row.type_ == "float" {
            let value = row.preview.parse::<f32>().unwrap_or(0.0);
            query = query.add_field(row.name, value);
        } else {
            let value = row.preview.to_string();
            query = query.add_field(row.name, value);
        }
    }

    query = query.add_field("ResponseTime", response_time as i64);
    result.push(query);

    Ok(result)
}
