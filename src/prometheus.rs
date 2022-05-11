use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use crate::spec::WriteSpec;
use crate::testconfig::ExportToPrometheus;
use influxdb::Type;
use lazy_static::lazy_static;
use prometheus::{GaugeVec, HistogramOpts, HistogramVec, Opts, Registry};
use tokio::sync::mpsc::Receiver;
use warp::{Filter, Rejection, Reply};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
}

pub async fn prometheus_thread(
    etp: ExportToPrometheus,
    mut receiver: Receiver<Vec<WriteSpec>>,
) -> anyhow::Result<()> {
    log::info!("Prometheus metric service initialization...");
    // HistogramVec, only response time
    let bucket_bin = etp.response_time_bucket_bin.clone();
    let opts =
        HistogramOpts::new("ResponseTime", "Response time in milliseconds").buckets(bucket_bin);
    let response_time = HistogramVec::new(opts, &["Service"])?;

    REGISTRY.register(Box::new(response_time.clone()))?;
    log::debug!("Response time metric registered.");

    let gauge_map: Arc<RwLock<HashMap<String, GaugeVec>>> = Arc::new(RwLock::new(HashMap::new()));

    // let counter_list = etp.counter_list.clone();

    log::info!("Lunching Prometheus metric service...");
    launch_prometheus_service(&etp).await?;

    loop {
        match receiver.recv().await {
            None => break,
            Some(write_specs) => {
                for write_spec in write_specs {
                    for field in write_spec.fields {
                        let name = format!("{}_{}", write_spec.measurement, field.0);
                        let value = match field.1 {
                            Type::Boolean(_) => {
                                continue;
                            }
                            Type::Float(value) => value,
                            Type::SignedInteger(value) => value as f64,
                            Type::UnsignedInteger(value) => value as f64,
                            Type::Text(_) => {
                                continue;
                            }
                        };

                        if field.0 == "ResponseTime" {
                            response_time
                                .with_label_values(&[&write_spec.measurement])
                                .observe(value / 1000000.0); // convert to milliseconds from nanoseconds
                        } else {
                            let map = gauge_map.read().expect("Read Lock poisoned.");
                            let mut label_values: Vec<&str> = vec![];
                            for tag in write_spec.tags.iter() {
                                match &tag.1 {
                                    Type::Text(value) => label_values.push(value),
                                    _ => {
                                        continue;
                                    }
                                }
                            }
                            if let Some(counter) = map.get(&name) {
                                counter.with_label_values(&label_values).set(value);
                            } else {
                                drop(map);
                                let mut map = gauge_map.write().expect("Write Lock poisoned.");
                                let mut label_names: Vec<&str> = vec![];
                                for tag in write_spec.tags.iter() {
                                    label_names.push(&tag.0);
                                }

                                let counter = GaugeVec::new(
                                    Opts::new(&name, &format!("{} Gauge", name)),
                                    &label_names,
                                )?;
                                REGISTRY.register(Box::new(counter.clone()))?;
                                counter.with_label_values(&label_values).set(value);
                                map.entry(name).or_insert_with(|| counter);
                            };
                        }
                    }
                }
            }
        }
    }
    todo!()
}

pub async fn launch_prometheus_service(etp: &ExportToPrometheus) -> anyhow::Result<()> {
    let metrics_route = warp::path!("metrics").and_then(metrics_handler);
    let addr = format!("{}:{}", "0.0.0.0", etp.port);
    log::info!("Prometheus metric service will be launched on {}", addr);
    let metrics_addr: SocketAddr = match addr.parse() {
        Ok(value) => value,
        Err(e) => {
            log::error!("Failed to parse:{} as addr,error:{:?}", addr, e);
            "0.0.0.0:19090".parse().unwrap()
        }
    };

    tokio::spawn(warp::serve(metrics_route).run(metrics_addr));
    log::info!("Prometheus metric HTTP service launched.");
    Ok(())
}
pub async fn metrics_handler() -> Result<impl Reply, Rejection> {
    let res = retrieve_metrics().await;
    Ok(res)
}

pub async fn retrieve_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    // if *load::INCLUDE_PROMETHEUS_METRICS {
    //     let mut buffer = Vec::new();
    //     if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
    //         eprintln!("could not encode prometheus metrics: {}", e);
    //     };
    //     let res_custom = match String::from_utf8(buffer.clone()) {
    //         Ok(v) => v,
    //         Err(e) => {
    //             eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
    //             String::default()
    //         }
    //     };
    //     buffer.clear();

    //     res.push_str(&res_custom);
    // }

    res
}
