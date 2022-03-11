use std::sync::{atomic::AtomicBool, Arc};

use crate::testconfig::TestConfig;
use crate::{influx::launch_influx_service, twxquery::launch_twxquery_service};
use tokio::sync::mpsc::channel;

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

    // launch influx service to store data first.
    let export_to_influxdb = tc.export_to_influxdb.clone();
    let export_to_file = tc.export_to_file.clone();
    let export_to_influxdb_task = tokio::spawn(async move {
        match launch_influx_service(&export_to_influxdb, receiver, export_to_file).await {
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
