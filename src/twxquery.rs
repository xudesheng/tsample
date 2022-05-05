use std::{
    collections::{BTreeMap, HashMap},
    time::SystemTime, sync::{atomic::{AtomicBool, Ordering}, Arc},
};

use crate::{
    testconfig::{SubSystem, TestConfig, ThingworxServer, ArbitraryMetric},
    payload::{TwxJson, ConnectionServerResults}, jmxquery::{ refresh_jmx},
};
use chrono::offset::Utc;
use chrono::DateTime;
use influxdb::{WriteQuery, Timestamp, InfluxDbWriteable};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use serde_json::Value as JsonValue;
use tokio::sync::mpsc::Sender;

pub async fn launch_twxquery_service(
    tc: TestConfig,
    sender: Sender<Vec<WriteQuery>>,
    running: Arc<AtomicBool>,
    sleeping: Arc<AtomicBool>,
    cxs_refresh_sleeping:  Arc<AtomicBool>,
    jmx_refresh_sleeping:  Arc<AtomicBool>,
) -> anyhow::Result<()> {
    // launch a new service to query all the connection servers under each Thingworx server if it is configured.
    // this map will hold (k,v) where k is the thingworx server name and v is the list of connection servers name under this thingworx server.
    let (cxserver_reader,mut cxserver_writer) = evmap::new();
    let mut need_refresh_connection_server = false;
    for server in tc.thingworx_servers.iter(){
        if let Some(ref cxserver_config) = server.connection_servers{
            if cxserver_config.names.is_empty(){
                log::info!("no connection server name defined under thingworx server:{:?}", server.name);
                need_refresh_connection_server = true;
            }
            cxserver_writer.insert(server.name.clone(), (cxserver_config.names.clone(),cxserver_config.metrics.clone()));
        }
    }
    cxserver_writer.refresh();
    if need_refresh_connection_server {
        log::info!("need to refresh connection server name");
        let tc_cxserver= tc.clone();
        tokio::spawn(async move {
            let _= refresh_connection_server(tc_cxserver,  cxserver_writer,cxs_refresh_sleeping).await;
        });
    }

    let (jmx_reader,mut jmx_writer) = evmap::new();
    let mut need_refresh_jmx = false;
    for server in tc.thingworx_servers.iter() {
        if let Some(ref jmx_config) = server.jmx_metrics {
            if !jmx_config.is_empty() {
                log::info!("JMX has been configured, JMX objectName needs to be refreshed from thingworx server:{:?}", server.name);
                need_refresh_jmx = true;
            }
            
        }
    }
    jmx_writer.refresh();
    if need_refresh_jmx {
        log::info!("need to refresh JMX object name");
        let tc_jmx = tc.clone();
        tokio::spawn(async move {
            let _= refresh_jmx(tc_jmx,  jmx_writer,jmx_refresh_sleeping).await;
        });
    }

    let scrap_interval = tc.scrap_interval as u64;
    let query_timeout = tc.query_time_out; //default should be 20 seconds
    log::info!("scrap interval is {} seconds, query timeout is:{} seconds.", scrap_interval, query_timeout);
    while running.load(Ordering::SeqCst) {
        let start_time = SystemTime::now();
        let mut tasks = vec![];
        for server in tc.thingworx_servers.iter() {
            let test_server = server.clone();
            let test_sender = sender.clone();
            // regular thingworx subsystem query
            let task = tokio::spawn(async move {
                match repeated_twxserver_query(&test_server, test_sender,query_timeout).await {
                    Ok(_) => {
                        // log::info!("twxquery service finished smoothly");
                    }
                    Err(e) => {
                        log::error!("twxquery service error:{:?}", e);
                    }
                }
            });

            tasks.push(task);

            // connection server query
            let local_cxserver_reader = cxserver_reader.clone();
            if let Some(_cxserver_config) = &server.connection_servers {
                if let Some(ref cache) = local_cxserver_reader.get_one(&server.name){
                    let names = cache.0.clone();
                    log::debug!("Server:{} has connection servers:{:?}", server.name, names);
                    for name in names {
                        let test_server = server.clone();
                        let test_sender = sender.clone();
                        let metrics = cache.1.clone();
                        let task = tokio::spawn(async move {
                            match repeated_connection_server_query(&test_server, &name, metrics, test_sender,query_timeout).await {
                                Ok(_) => {
                                    // log::info!("connection server query finished smoothly");
                                }
                                Err(e) => {
                                    log::error!("connection server query error:{:?}", e);
                                }
                            }
                        });
                        tasks.push(task);
                    }
                }
            }

            // c3p0 query
            let local_jmx_reader = jmx_reader.clone();
            if let Some(_jmx_config) = &server.jmx_metrics {
                if let Some(ref cache) = local_jmx_reader.get_one(&server.name){
                    // let jmx_metrics_vec = cache.clone();
                    log::debug!("Server:{} has jmx metrics:{:?}", server.name, cache);
                    for (measurement, object_name_list, name_alternative, metrics) in cache.iter() {
                        let test_server = server.clone();
                        let test_sender = sender.clone();
                        let measurement = measurement.clone();
                        let object_name_list = object_name_list.clone();
                        let name_alternative = name_alternative.clone();
                        let metrics = metrics.clone();
                        let task = tokio::spawn(async move {
                            match crate::jmxquery::repeated_jmx_query(&test_server,
                                 measurement,
                                 object_name_list,
                                 name_alternative,
                                 metrics, 
                                 test_sender,query_timeout).await {
                                Ok(_) => {
                                    // log::info!("connection server query finished smoothly");
                                }
                                Err(e) => {
                                    log::error!("JMX query error:{:?}", e);
                                }
                            }
                        });
                        tasks.push(task);
                    }
                }
            }

            // arbitrary url metrics query
            if let Some(ref arbitrary_config) = &server.arbitrary_metrics {
                for am in arbitrary_config.iter() {
                    if !am.enabled {
                        continue;
                    }
                    let test_server = server.clone();
                    let test_sender = sender.clone();
                    let am = am.clone();
                    let task = tokio::spawn(async move {
                        match repeated_arbitrary_query(&test_server, am, test_sender,query_timeout).await {
                            Ok(_) => {
                                // log::info!("connection server query finished smoothly");
                            }
                            Err(e) => {
                                log::error!("arbitrary url query error:{:?}", e);
                            }
                        }
                    });
                    tasks.push(task);
                }
            }
        }
        for task in tasks {
            let _ = task.await?;
        }
        if !running.load(Ordering::SeqCst) {
            break;
        }
        let spent_time = SystemTime::now().duration_since(start_time).unwrap();
        
        if scrap_interval * 1000 > spent_time.as_millis() as u64 {
            let sleep_time = scrap_interval * 1000 - spent_time.as_millis() as u64;
            log::info!("sleep {} seconds", sleep_time/1000);
            sleeping.store(true, Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_millis(sleep_time)).await;
            sleeping.store(false, Ordering::SeqCst);
        }
    }
    
    Ok(())
}

pub async fn repeated_arbitrary_query(
    server: &ThingworxServer,
    am: ArbitraryMetric,
    sender: Sender<Vec<WriteQuery>>,
    query_timeout: u64,
)->anyhow::Result<()>{
    let url = server.get_arbitrary_access_url(&am.url);
    let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(query_timeout))
        .danger_accept_invalid_certs(true)
        .build()?;
    log::debug!("Arbitrary metrics query service url:{}", url);
    let headers = construct_headers(&server.app_key);
    let metrics_name = am.name.clone();
    let am_subsystem = SubSystem{
        name: am.name,
        options: am.options,
        enabled: am.enabled,
        sanitize: am.sanitize,
        split_desc_asprefix: am.split_desc_asprefix
    };

    let result = match query_subsystem_metrics(client, &url, &headers, &am_subsystem,&server.name,None).await{
        Ok(result) => result,
        Err(e) => {
            log::error!("Arbitrary metrics query service error:{:?}", e);
            return Ok(());
        }
    };
    log::debug!("Arbitrary metrics:{} metrics result:{}",metrics_name, result.len());
    let _ = sender.send(result).await;

    Ok(())
}
pub async fn repeated_connection_server_query(
    server:&ThingworxServer,
    cxserver_name:&str,
    metrics:Vec<String>,
    sender:Sender<Vec<WriteQuery>>,
    query_timeout: u64,
)->anyhow::Result<()>{
    let url = server.get_cxserver_query_service_url(cxserver_name);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(query_timeout))
        // This is supported on crate feature native-tls only
        // .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    log::debug!("Connection Server query service url:{}", url);
    let headers = construct_headers(&server.app_key);
    let cx_subsystem = SubSystem{
        name: "ConnectionServer".to_string(),
        options: Some(metrics),
        enabled: true,
        sanitize: true,
        split_desc_asprefix: false,
    };
    let mut additional_tags=HashMap::new();
    additional_tags.insert("cxserver".to_string(), cxserver_name.to_string());

    let result = match query_subsystem_metrics(client, &url, &headers, &cx_subsystem,&server.name,Some(additional_tags)).await{
        Ok(result) => result,
        Err(e) => {
            log::error!("query connection server metrics error:{:?}", e);
            return Ok(());
        }
    };
    log::debug!("query connection server:{} metrics result:{}",cxserver_name, result.len());
    let _ = sender.send(result).await;
    Ok(())

}

pub async fn refresh_connection_server(
    tc:TestConfig,
    // reader: &evmap::ReadHandle<String, (Vec<String>,Vec<String>)>,
    mut writer: evmap::WriteHandle<String, (Vec<String>,Vec<String>)>,
    sleeping: Arc<AtomicBool>,
)->anyhow::Result<()>{
    
    loop{
        for server in tc.thingworx_servers.iter(){
            if let Some(ref cxserver_config) = server.connection_servers{
                // if it has a name list configured, then we ignore it.
                if !cxserver_config.names.is_empty(){
                    continue;
                }
                let headers = construct_headers(&server.app_key);
                let url = server.get_cxserver_query_url();
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(20))
                    .danger_accept_invalid_certs(true)
                    .build()?;
                log::debug!("connection server query service url:{}", url);
                let res = match client.post(url).headers(headers.clone()).send().await{
                    Ok(res) => res,
                    Err(e) => {
                        log::error!("connection server query service error:{:?}", e);
                        continue;
                    }
                };
                if !res.status().is_success() {
                    log::error!("connection server query :{} failed", server.name);
                    continue;
                }
            
                let cxservers: ConnectionServerResults = match res.json::<ConnectionServerResults>().await {
                    Ok(cxservers) => cxservers,
                    Err(e) => {
                        log::error!(
                            "connection server query :{} failed to parse result:{:?}",
                            server.name,
                            e
                        );
                        continue;
                    }
                };
                let mut names = vec![];
                for row in cxservers.rows{
                    names.push(row.name);
                }
                
                log::info!("connection server query :{} success, got {} connection server(s)", server.name, names.len());
                writer.update(server.name.clone(), (names,cxserver_config.metrics.clone()));
            }
        }
        writer.refresh();
        sleeping.store(true, Ordering::SeqCst);
        tokio::time::sleep(std::time::Duration::from_secs(tc.refresh_server_interval )).await;
        sleeping.store(false, Ordering::SeqCst);
    }
    // Ok(())
}
pub async fn repeated_twxserver_query(
    server: &ThingworxServer,
    sender: Sender<Vec<WriteQuery>>,
    query_timeout:u64,
) -> anyhow::Result<()> {
    let url = format!("{}://{}:{}", server.protocol, server.host, server.port);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(query_timeout))
        // This is supported on crate feature native-tls only
        // .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()?;
    log::debug!("twxquery service url:{}", url);
    let headers = construct_headers(&server.app_key);
    for subsystem in server.subsystems.iter() {
        if !subsystem.enabled {
            continue;
        }
        let sys_url = format!(
            "{}/{}/Subsystems/{}/Services/GetPerformanceMetrics",
            url, server.application, subsystem.name
        );
        match query_subsystem_metrics(client.clone(), &sys_url, &headers, subsystem, &server.name,None).await {
            Ok(metrics) => {
                log::debug!("result from subsystem:{} has:{} metrics", subsystem.name, metrics.len());
                let _ = sender.send(metrics).await;
            }
            Err(e) => {
                log::error!("query subsystem metrics error:{:?}", e);
                break;
            }
        }
    }
    Ok(())
}

pub fn construct_headers(app_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    // impossible to fail here
    headers.insert("appKey", HeaderValue::from_str(app_key).unwrap());
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    headers
}

pub async fn query_subsystem_metrics(
    client: Client,
    url: &str,
    headers: &HeaderMap,
    subsystem: &SubSystem,
    platform:&str,
    additional_tags:Option<HashMap<String,String>>,
) -> anyhow::Result<Vec<WriteQuery>> {
    let mut result = vec![];
    let response_start = SystemTime::now();
    
    // if this step is error, likely the server is not responsive.
    // so we should not continue to query the metrics.
    // for the rest of the metrics, we will just handle the error within this block.
    let res = client.post(url).headers(headers.clone()).send().await?;
    if !res.status().is_success() {
        // return Err(anyhow::anyhow!("Subsystem metrics query:{} failed", url));
        log::error!("Subsystem metrics query:{} failed", url);
        return Ok(result); // return empty result
    }

    let twx_json: TwxJson = match res.json::<TwxJson>().await {
        Ok(twx_json) => twx_json,
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

    let mut metric_value_map: HashMap<String, BTreeMap<String, JsonValue>> = HashMap::new();
    let system_time = SystemTime::now();
    let timestamp: DateTime<Utc> = system_time.into();
    // we can consume all rows here.
    for row in twx_json.rows {
        // only process metrics if the subsystem has limited options for metrics names.
        if let Some(ref options) = subsystem.options {
            if !options.is_empty() &&  !options.contains(&row.name) {
                continue;
            }
        }
        if row.value.is_none() {
            continue;
        }
        let row_desc = row.description.unwrap_or_else(|| "".to_string());
        let row_value = row.value.unwrap(); //it's safe

        // Persistent Provider needs to be handled differently
        let (provider, _description) = match row_desc.find(": ") {
            Some(start) => (
                (&row_desc[0..start]).to_string(),
                (&row_desc[start + 2..]).to_string(),
            ),
            // common metrics will use 'default' value for provider
            None => ("Default".to_string(), (&row_desc).to_string()),
        };
        if !metric_value_map.contains_key(&provider) {
            let value_map: BTreeMap<String, JsonValue> = BTreeMap::new();
            metric_value_map.insert(provider.clone(), value_map);
        }

        if let Some(value_map) = metric_value_map.get_mut(&provider) {
            //value_map.insert(row.name.clone(), row.value.clone());
            value_map.insert(
                // too many redundant letters in the name of the metric from the connection server.
                // we can optimize it future by shorting the name.
                sanitize_name(&row.name, subsystem.sanitize),//
                // row.name,
                row_value,
            );
        }
    }

    for (provider, value_map) in &metric_value_map {
        if value_map.is_empty() {
            continue;
        }
        let mut query = Timestamp::Milliseconds(timestamp.timestamp_millis().try_into().unwrap()).into_query(&subsystem.name)
            .add_tag("Provider", provider.to_string())
            .add_tag("Platform", platform.to_string()) 
            ;
        if let Some(ref additional_tags) = additional_tags {
            // Metrics from all connection servers will be in a dummy subsystem 'ConnectionServer'.
            // Therefore, it requires an additional tag to be added, which is the connection server name.
            for (key, value) in additional_tags {
                // this can be optimized in future to avoid copy.
                // it should directly consume the hashmap.
                query = query.add_tag(key.clone(), value.clone());
            }
        }
        for (key, value) in value_map {
            match value{
                JsonValue::Number(num)=>{
                    match num.as_f64() {
                        Some(num) =>{
                            query = query.add_field(key, num);
                            
                        },
                        None=>{
                            query = query.add_field(key, 0.0_f64);
                            
                        },
                    }
                }
                JsonValue::Null => {}
                JsonValue::Bool(boolvalue) => {
                    query = query.add_field(key, *boolvalue);
                    }
                JsonValue::String(strvalue) => {
                    query = query.add_field(key, strvalue.to_string());
                    }
                JsonValue::Array(value) => {
                    match serde_json::to_string(value){
                        Ok(strvalue)=>{query = query.add_field(key, strvalue);},
                        Err(e)=>log::error!("Failed to convert array result to string:key:{},value:{:?},error:{:?}",key,value,e),
                    }
                    
                }
                JsonValue::Object(value) => {
                    match serde_json::to_string(value){
                        Ok(strvalue)=>{query = query.add_field(key, strvalue);},
                        Err(e)=>log::error!("Failed to convert object map result to string:key:{},value:{:?},error:{:?}",key,value,e),
                    }
                }
            }
            
        }

        query = query.add_field("ResponseTime", response_time as i64);
        result.push(query);
    }
    Ok(result)
}


fn sanitize_name(name:&str,sanizie:bool)->String {
    if sanizie {
        let mut r = String::with_capacity(name.len());
        for (_i,d) in name.char_indices(){
            if "\"'.- ()/".contains(d) {
                r.push('_');
            }else{
                r.push(d);
            }
        }
        return r;
    }

    name.to_string()
}
