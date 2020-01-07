extern crate reqwest;

use influx_db_client::{Point, Value};
use crate::thingworxtestconfig::{ThingworxServer, RepeatTest, OneTimeTest};
use crate::thingworxjson::*;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::Client as ReqClient;
use std::time::Duration;
use sys_info::*;
use std::error::Error;
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::OpenOptions;
use std::io::LineWriter;
use std::path::Path;
use std::io::Write;
use chrono::offset::Utc;
use chrono::DateTime;

pub fn samping_one_time(testid: &str, o_sampling: &OneTimeTest) -> Result<Point, Box<dyn Error>>{
    //meansurement name
    let mut point = Point::new("onetime_sampling");

    point.add_tag("testid", Value::String(testid.to_string()));
    if o_sampling.os_type {point.add_field("os_type", Value::String(match os_type(){ Ok(value)=>value,Err(_)=>"Unknown".to_string(),}));}
    if o_sampling.cpu_num {point.add_field("cpu_num", Value::Integer(match cpu_num(){ Ok(value)=>value as i64,Err(_)=>0 as i64,}));}
        //.add_field("proc_total",Value::String(match proc_total(){ Ok(value)=>value,Err(_)=>"Unknown",}))
    if o_sampling.cpu_speed {point.add_field("cpu_speed", Value::Integer(match cpu_speed(){ Ok(value)=>value as i64,Err(_)=>0 as i64,}));}
    if o_sampling.hostname {point.add_field("hostname", Value::String(match hostname(){ Ok(value)=>value,Err(_)=>"Unknown".to_string()}));}

    let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

    point.add_timestamp(timestamp.as_millis() as i64);
    debug!("{:?}", point);
    Ok(point)
}

pub fn sampling_repeat(testid: &str, r_sampling: &RepeatTest, export_path:&Path, export_file:bool) -> Result<Point, Box<dyn Error>>{
    let mut point = Point::new("repeat_sampling");
    point.add_tag("testid", Value::String(testid.to_string()));
    
    //let r_sampling = &test_config.testmachine.repeat_sampling;

    let load = loadavg()?;
    if r_sampling.mem_info_one {point.add_field("mem_info_one", Value::Float(load.one));}
    if r_sampling.mem_info_five {point.add_field("mem_info_five", Value::Float(load.five));}
    if r_sampling.mem_info_fifteen {point.add_field("mem_info_fifteen", Value::Float(load.fifteen));}

    let mem = mem_info()?;
    if r_sampling.mem_total {point.add_field("mem_total",Value::Integer(mem.total as i64));}
    if r_sampling.mem_free {point.add_field("mem_free",Value::Integer(mem.free as i64));}
    if r_sampling.mem_avail {point.add_field("mem_avail",Value::Integer(mem.avail as i64));}
    if r_sampling.mem_buffers {point.add_field("mem_buffers",Value::Integer(mem.buffers as i64));}
    if r_sampling.mem_cached {point.add_field("mem_cached",Value::Integer(mem.cached as i64));}
    if r_sampling.swap_total {point.add_field("swap_total",Value::Integer(mem.swap_total as i64));}
    if r_sampling.swap_free {point.add_field("swap_free",Value::Integer(mem.swap_free as i64));}
    
    let disk = disk_info()?;
    if r_sampling.disk_total {point.add_field("disk_total", Value::Integer(disk.total as i64));}
    if r_sampling.disk_free {point.add_field("disk_free", Value::Integer(disk.free as i64));}

    let proc_total = proc_total()?;
    if r_sampling.proc_total {point.add_field("proc_total", Value::Integer(proc_total as i64));}

    let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

    point.add_timestamp(timestamp.as_millis() as i64);
    let point = point.to_owned();
    debug!("sampling_repeat: {:?}", point);

    if export_file {
        let export_file = export_path.join("system_records.csv");
        let mut export_header: bool = false;
        if !export_file.exists() {
            export_header = true;
        }

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(export_file)?;

        let mut export_file = LineWriter::new(file);
        if export_header {
            const HEADER: &str = "timestamp,cpu_info_one,cpu_info_five,cpu_info_fifteen,mem_total,\
            mem_free,mem_avail,mem_buffers,mem_cached,swap_total,swap_free,disk_total,disk_free,\
            proc_total\n";
            export_file.write_all(HEADER.as_bytes())?;
        }
        let system_time = SystemTime::now();
        let datetime: DateTime<Utc> = system_time.into();

        let data = format!("{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",datetime.format("%Y-%m-%d %H:%M:%S"),load.one,load.five,load.fifteen,
                mem.total, mem.free, mem.avail, mem.buffers,mem.cached,
                mem.swap_total,mem.swap_free,disk.total,disk.free,proc_total);
        export_file.write_all(data.as_bytes())?;
        export_file.flush()?;
    }

    Ok(point)
}

fn construct_headers(app_key:&str) -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert("appKey", HeaderValue::from_str(app_key)?);
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    Ok(headers)
}

pub fn sampling_thingworx(twx_server: &ThingworxServer, export_path:&Path, export_file:bool) -> Result<Vec<Point>, Box<dyn Error>>{
    
    //let client = ReqClient::new();
    let client = ReqClient::builder()
                    .gzip(true)
                    .timeout(Duration::from_secs(10))
                    .build()?;
    
    let mut points:Vec<Point> = Vec::new();
    let mut export_points: Vec<BTreeMap<String,String>> = Vec::new();

    for metric in twx_server.metrics.iter() {
        if !metric.enabled {continue;}
        let url = twx_server.get_url()?;
        let url = format!("{}/{}", url, metric.url);
        //println!("url:{}", url);
        let headers = construct_headers(&twx_server.app_key)?;
        //println!("header:{:?}", headers);
        //let mut res = client.post(&url).headers(headers).send()?;

        match client.post(&url).headers(headers).send() {
            Ok(mut res) => {
                if res.status().is_success(){
                    if let Ok(twx_json) = res.json::<TwxJson>(){
                        //good points after parsing (deserialization)
                        debug!("JSON:{:?}", twx_json);
                        let mut points_map:HashMap<String, Option<Point>>=HashMap::new();
                        let mut export_points_map:HashMap<String,Option<BTreeMap<String,String>>>=HashMap::new();

                        let mut key_list = Vec::new();

                        for row in twx_json.rows.iter(){
                            let (key,description) = match row.description.find(": "){
                                Some(start) => ((&row.description[0..start]).to_string(),(&row.description[start+2..]).to_string()),
                                None => ("Default".to_string(), (&row.description).to_string()),
                            };

                            if !points_map.contains_key(&key) {
                                let measurement = format!("{}_{}", &metric.name, &row.name);
                                let mut point = Point::new(&measurement);
                                let mut hm:BTreeMap<String,String> = BTreeMap::new();

                                hm.insert("Measurement".to_string(), measurement.clone());
                                hm.insert("Provider".to_string(), key.clone());
                                point.add_tag("Provider", Value::String(key.to_string()));
                                point.add_tag("description",Value::String(description.clone()));
                                let host = match &twx_server.alias {
                                    Some(alias)=> format!("{}_{}",alias, &twx_server.host),
                                    None => format!("{}", &twx_server.host),
                                };

                                point.add_tag("host",Value::String(host));
                                point.add_tag("component", Value::String("platform_thingworx".to_string()));

                                hm.insert("QUALITY".to_string(), "GOOD".to_string() );
                                point.add_tag("QUALITY", Value::String("GOOD".to_string()));

                                hm.insert("STATUS".to_string(),res.status().to_string().clone());
                                point.add_tag("STATUS", Value::String(res.status().to_string()));

                                let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

                                point.add_timestamp(timestamp.as_millis() as i64);
                                point.add_field("value", Value::Float(row.value));
                                let system_time = SystemTime::now();
                                let datetime: DateTime<Utc> = system_time.into();
                                hm.insert("TIMESTAMP".to_string(),datetime.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string());

                                points_map.insert(key.to_string(), Some(point));
                                export_points_map.insert(key.to_string(), Some(hm));

                                key_list.push(key.clone());
                            }

                            if let Some(content) = points_map.get_mut(&key){
                                match content {
                                    Some(point)=>{point.add_field(&row.name, Value::Float(row.value));},
                                    None => unreachable!(),
                                }
                                
                            }

                            if let Some(content) = export_points_map.get_mut(&key){
                                match content {
                                    Some(hashmap) => {hashmap.insert(row.name.clone(),format!("{}",row.value));},
                                    None => unreachable!(),
                                }
                            }

                        }

                        for old_key in key_list.iter(){
                            match points_map.remove(&old_key.to_string()) {
                                Some(value)=> {
                                    match value {
                                        Some(point) => points.push(point),
                                        None => unreachable!(),
                                    }
                                },
                                None => unreachable!(),
                            }

                            match export_points_map.remove(&old_key.to_string()) {
                                Some(value) => {
                                    match value {
                                        Some(hashmap) => export_points.push(hashmap),
                                        None => unreachable!(),
                                    }
                                },
                                None => unreachable!(),
                            }
                        }

                    }else{
                        //bad JSON response.
                        let mut point = Point::new(&metric.name);

                        let mut hm:BTreeMap<String,String> = BTreeMap::new();

                        hm.insert("Measurement".to_string(), "Default".to_string());

                        hm.insert("Provider".to_string(),"Default".to_string());
                        point.add_tag("Provider", Value::String("Default".to_string()));

                        hm.insert("QUALITY".to_string(), "BADJSON".to_string() );
                        point.add_field("QUALITY", Value::String("BADJSON".to_string()));

                        hm.insert("STATUS".to_string(),res.status().to_string().clone());
                        point.add_field("STATUS", Value::String(res.status().to_string()));
                        let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

                        point.add_timestamp(timestamp.as_millis() as i64);

                        let system_time = SystemTime::now();
                        let datetime: DateTime<Utc> = system_time.into();
                        hm.insert("TIMESTAMP".to_string(),datetime.format("%Y-%m-%d %H:%M:%S").to_string());
                        export_points.push(hm);
                        points.push(point);
                    }
                }else{
                    //bad status (not success.)
                    let mut point = Point::new(&metric.name);
                    let mut hm:BTreeMap<String,String> = BTreeMap::new();

                    hm.insert("Measurement".to_string(), "Default".to_string());
                    hm.insert("Provider".to_string(),"Default".to_string());
                    hm.insert("QUALITY".to_string(), "BADSTATUS".to_string() );
                    hm.insert("STATUS".to_string(),res.status().to_string().clone());

                    point.add_tag("Provider", Value::String("Default".to_string()));
                    point.add_field("QUALITY", Value::String("BADSTATUS".to_string()));
                    point.add_field("STATUS", Value::String(res.status().to_string()));
                    
                    let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

                    point.add_timestamp(timestamp.as_millis() as i64);

                    let system_time = SystemTime::now();
                    let datetime: DateTime<Utc> = system_time.into();
                    hm.insert("TIMESTAMP".to_string(),datetime.format("%Y-%m-%d %H:%M:%S").to_string());

                    export_points.push(hm);
                    points.push(point);
                }
            },
            Err(e) => {
                //failed http call.
                let mut point = Point::new(&metric.name);
                point.add_tag("Provider", Value::String("Default".to_string()));
                point.add_field("QUALITY", Value::String("WRONG".to_string()));
                point.add_field("STATUS", Value::String("-1".to_string()));
                
                let mut hm:BTreeMap<String,String> = BTreeMap::new();

                hm.insert("Measurement".to_string(), "Default".to_string());
                hm.insert("Provider".to_string(),"Default".to_string());
                hm.insert("QUALITY".to_string(), "WRONG".to_string() );
                hm.insert("STATUS".to_string(),"-1".to_string());
                let system_time = SystemTime::now();
                let datetime: DateTime<Utc> = system_time.into();
                hm.insert("TIMESTAMP".to_string(),datetime.format("%Y-%m-%d %H:%M:%S").to_string());

                export_points.push(hm);
                
                let timestamp=SystemTime::now().duration_since(UNIX_EPOCH)?;

                point.add_timestamp(timestamp.as_millis() as i64);
                points.push(point);
                debug!("HTTP Error:{}", e);
            }
        }
    }
    //unimplemented!()
    if export_file {
        for export_point in export_points{
            let filename=format!("{}_{}.txt", export_point.get("Measurement").or(Some(&"Default".to_string())).unwrap(),
                    export_point.get("Provider").or(Some(&"Default".to_string())).unwrap()
                );
            let export_file = export_path.join(filename);
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(export_file)?;

            let mut export_file = LineWriter::new(file);
            
            let data = format!("{:?}\n",export_point);
            export_file.write_all(data.as_bytes())?;
            export_file.flush()?;
        }
        
    }

    debug!("{:?}", points);
    Ok(points)
}

//Ok(())

    // println!("Configuration:\n{:?}", testconfig);

    // println!("os: {} {}", os_type().unwrap(), os_release().unwrap());
    // println!("cpu: {} cores, {} MHz", cpu_num().unwrap(), cpu_speed().unwrap());
    // println!("proc total: {}", proc_total().unwrap());
    // let load = loadavg().unwrap();
    // println!("load: {} {} {}", load.one, load.five, load.fifteen);
    // let mem = mem_info().unwrap();
    // println!("mem: total {} KB, free {} KB, avail {} KB, buffers {} KB, cached {} KB",
    //          mem.total, mem.free, mem.avail, mem.buffers, mem.cached);
    // println!("swap: total {} KB, free {} KB", mem.swap_total, mem.swap_free);
    // let disk = disk_info().unwrap();
    // println!("disk: total {} KB, free {} KB", disk.total, disk.free);
    // println!("hostname: {}", hostname().unwrap());
    // let t = boottime().unwrap();
    // println!("boottime {} sec, {} usec", t.tv_sec, t.tv_usec);

    // let client = Client::new();
    // let mut res = client.post("https://twx85.desheng.io/Thingworx/Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics")
    //     .headers(construct_headers())
    //     .send()?;
    // if res.status().is_success(){
    //     println!("Success!");

    //     if let Ok(twx_json) = res.json::<TwxJson>(){
    //         println!("parsed successfully.");
    //         //let rows = twx_json.rows;
    //         //println!("{:#?}", twx_json.rows);
    //         for row in twx_json.rows.iter(){
    //             //println!("{:#?}", row);
    //             let key=match row.description.find(": ") {
    //                 Some(start) => &row.description[0..start],
    //                 None => "Default",
    //             };
    //             println!("key:{},name:{}, \n\tvalue:{}\n\tDesc:{}", key,row.name, row.value, row.description);
    //         }
    //     }else{
    //         println!("wrong parsing");
    //     }
    //     // let mut res_body = String::new();
    //     // res.read_to_string(&mut res_body)?;
    //     // println!("result: {}", res_body);

    // }else if res.status().is_server_error(){
    //     println!("Server error");
    // }else{
    //     println!("unknown status:{}", res.status());
    // }

    