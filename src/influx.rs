use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use crate::spec::WriteSpec;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use influxdb::WriteQuery;
use influxdb::{Client, InfluxDbWriteable};
use tokio::sync::mpsc::Receiver;

use crate::testconfig::{ExportToFile, ExportToInfluxDB};

pub async fn launch_influx_service(
    influx_config: &ExportToInfluxDB,
    mut receiver: Receiver<Vec<WriteSpec>>,
    file_config: Option<ExportToFile>,
) -> anyhow::Result<()> {
    let enabled = influx_config.enabled;
    log::info!("influxdb service enabled:{}", enabled);
    let url = format!(
        "{}://{}:{}",
        influx_config.protocol, influx_config.server_name, influx_config.port
    );
    let client = Client::new(&url, influx_config.database.clone());
    let client = match (
        influx_config.username.as_ref(),
        influx_config.password.as_ref(),
    ) {
        (Some(username), Some(password)) => client.with_auth(username.clone(), password.clone()),
        _ => {
            log::debug!("no username and password");
            client
        }
    };

    let sender = match file_config {
        None => None,
        Some(file_config) => {
            if file_config.enabled {
                let (sender, receiver) = tokio::sync::mpsc::channel(1000);
                tokio::spawn(async move {
                    log::info!("Export to file service launched.");
                    match launch_file_service(file_config, receiver).await {
                        Ok(_) => {
                            log::info!("file service finished.");
                        }
                        Err(e) => {
                            log::error!("file service error:{:?}", e);
                        }
                    }
                });
                Some(sender)
            } else {
                None
            }
        }
    };
    loop {
        match receiver.recv().await {
            None => break,
            Some(write_spec) => {
                let mut write_query = vec![];
                for spec in write_spec {
                    let WriteSpec {
                        fields,
                        tags,
                        measurement,
                        timestamp,
                    } = spec;
                    let mut one_query = timestamp.into_query(measurement);

                    for field in fields {
                        one_query = one_query.add_field(field.0, field.1);
                    }
                    for tag in tags {
                        one_query = one_query.add_tag(tag.0, tag.1);
                    }
                    write_query.push(one_query);
                }

                if let Some(ref sender) = sender {
                    // 99.99% chance InfluxDB will be enabled, so, doesn't matter to clone the data.
                    let file_query = write_query.clone();
                    let _ = sender.send(file_query).await;
                }
                if enabled {
                    for query in write_query {
                        let _ = client.query(query).await;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn launch_file_service(
    file_config: ExportToFile,
    mut receiver: Receiver<Vec<WriteQuery>>,
) -> anyhow::Result<()> {
    let path = Path::new(&file_config.directory);
    if !(path.is_dir() && path.exists()) {
        if file_config.auto_create_folder {
            std::fs::create_dir_all(&file_config.directory)?;
        } else {
            return Err(anyhow::anyhow!("directory not exist"));
        }
    }
    let now = Utc::now();
    let mut baseline = get_datetime(now);
    let mut file_name = get_filename(now);

    let export_file_base = PathBuf::from(&file_config.directory);
    let mut export_file = export_file_base.clone();
    export_file.push(file_name.clone());
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(export_file)?;

    loop {
        match receiver.recv().await {
            None => break,
            Some(write_query) => {
                // check file name should be changed or not.
                let now = Utc::now();
                let new_base = get_datetime(now);
                if new_base != baseline {
                    baseline = new_base;
                    file_name = get_filename(now);

                    export_file = export_file_base.clone();
                    export_file.push(file_name.clone());
                    file = fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(export_file)?;
                }
                for query in write_query {
                    let content = format!("{},{:?}\n", now, query);
                    file.write_all(content.as_bytes())?;
                }
            }
        }
    }
    Ok(())
}

fn get_datetime(now: DateTime<Utc>) -> u32 {
    let newtime = NaiveDate::from_ymd(now.year(), now.month(), now.day()).and_hms(0, 0, 0);
    newtime.timestamp() as u32
}

fn get_filename(now: DateTime<Utc>) -> String {
    format!("metrics-{}-{}-{}.txt", now.year(), now.month(), now.day())
}
