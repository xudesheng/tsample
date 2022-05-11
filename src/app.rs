use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    influx::launch_influx_service, prometheus::prometheus_thread, twxquery::launch_twxquery_service,
};
use crate::{spec::WriteSpec, testconfig::TestConfig};
use tokio::sync::mpsc::{channel, Sender};

pub async fn run_app(
    tc: TestConfig,
    running: Arc<AtomicBool>,
    main_query_sleeping: Arc<AtomicBool>,
    cxs_refresh_sleeping: Arc<AtomicBool>,
    jmx_refresh_sleeping: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    if let Some(ref owner) = tc.owner {
        log::info!("test owner:{:?}", owner);
    }

    let (sender, receiver) = channel(1000);

    let prometheus_sender: Option<Sender<Vec<WriteSpec>>> =
        if let Some(ref prometheus_config) = tc.export_to_prometheus {
            let etp = prometheus_config.clone();

            let enabled = etp.enabled;
            if enabled {
                let (prom_sender, prom_receiver) = channel(1000);
                tokio::spawn(async move { prometheus_thread(etp, prom_receiver).await });
                Some(prom_sender)
            } else {
                None
            }
        } else {
            None
        };
    // launch influx service to store data first.
    let export_to_influxdb = tc.export_to_influxdb.clone();
    let export_to_file = tc.export_to_file.clone();
    let export_to_influxdb_task = tokio::spawn(async move {
        match launch_influx_service(
            &export_to_influxdb,
            receiver,
            export_to_file,
            prometheus_sender,
        )
        .await
        {
            Ok(_) => {
                log::info!("influxdb service finished smoothly");
            }
            Err(e) => {
                log::error!("influxdb service error:{:?}", e);
            }
        }
    });

    let query_running = running.clone();
    let twx_query_task = tokio::spawn(async move {
        match launch_twxquery_service(
            tc.clone(),
            sender,
            query_running,
            main_query_sleeping,
            cxs_refresh_sleeping,
            jmx_refresh_sleeping,
        )
        .await
        {
            Ok(_) => {
                log::info!("twxquery service finished.");
            }
            Err(e) => {
                log::error!("twxquery service error:{:?}", e);
            }
        }
    });

    let _ = twx_query_task.await;
    let _ = export_to_influxdb_task.await;

    Ok(())
}
