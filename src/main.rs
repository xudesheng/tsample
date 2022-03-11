mod app;
mod influx;
mod jmxquery;
mod payload;
mod testconfig;
mod twxquery;

use std::{
    io::Write,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use std::{env, fs::File};
use testconfig::TestConfig;

use clap::{Arg, Command};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const PKGNAME: &str = env!("CARGO_PKG_NAME");
static SAMPLE_CONFIG: &str = include_str!("../config/config.yaml");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_level = match env::var("TSAMPLE_LOG") {
        Ok(level) => level,
        Err(_) => "info".to_string(),
    };

    env_logger::Builder::new().parse_filters(&log_level).init();

    log::info!("{}:{} Started.", PKGNAME, VERSION);
    log::info!(
        "Log level: {}, you can change it by setting TSAMPLE_LOG env.",
        log_level
    );

    let matches = Command::new(PKGNAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("CONFIG_FILE")
                .help("Configuration file name, it should be a YAML file.")
                .takes_value(true),
        )
        .arg(
            Arg::new("export")
                .help("Export sample configuration file.")
                .short('e')
                .long("export")
                .requires("config"),
        )
        .arg(
            Arg::new("flatten")
                .help("Flatten the configuration file into a new yaml file for validation.")
                .short('f')
                .long("flatten")
                .value_name("FLATTEN_FILE")
                .requires("config"),
        )
        .get_matches();
    let config_file = match matches.value_of("config") {
        Some(value) => value.to_string(),
        None => env::var("TSAMPLE_CONFIG").unwrap_or_else(|_| "./config.yaml".to_string()),
    };

    if matches.is_present("export") {
        let mut output = File::create(&config_file)?;
        write!(output, "{}", SAMPLE_CONFIG)?;
        log::info!("Sample configuration file is exported to {}", config_file);
        return Ok(());
    }
    //tsample::ThingworxTestConfig::export_sample(config_file)?;
    if let Some(flatten_file) = matches.value_of("flatten") {
        let output = File::create(flatten_file)?;
        let config = TestConfig::load_from_file(&config_file)?;
        serde_yaml::to_writer(output, &config)?;

        log::info!(
            "Flattened configuration file is exported to {}",
            flatten_file
        );
        return Ok(());
    }
    let testconfig: TestConfig = TestConfig::load_from_file(&config_file)?;

    let running = Arc::new(AtomicBool::new(true));
    let main_query_sleeping = Arc::new(AtomicBool::new(false));
    let cxs_refresh_sleeping = Arc::new(AtomicBool::new(false));
    let jmx_refresh_sleeping = Arc::new(AtomicBool::new(false));

    let r = running.clone();
    let m = main_query_sleeping.clone();
    let c = cxs_refresh_sleeping.clone();
    let j = jmx_refresh_sleeping.clone();

    ctrlc::set_handler(move || {
        log::info!("Received Ctrl-C from console.");
        r.store(false, Ordering::SeqCst);
        if m.load(Ordering::SeqCst) || c.load(Ordering::SeqCst) || j.load(Ordering::SeqCst) {
            log::warn!("Quit from sleeping...");
            process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    app::run_app(
        testconfig,
        running,
        main_query_sleeping,
        cxs_refresh_sleeping,
        jmx_refresh_sleeping,
    )
    .await?;
    // let sleep = match testconfig.testmachine.sampling_cycle_inseconds {
    //     Some(seconds) => seconds * 1000,
    //     None => 120 * 1000,
    // };
    // let sleep_duration = time::Duration::from_millis(sleep);

    // let sampling_timeout_inseconds = match testconfig.testmachine.sampling_timeout_inseconds {
    //     Some(inseconds) => inseconds,
    //     None => 10,
    // };

    // let running = Arc::new(AtomicBool::new(true));
    // let sleeping = Arc::new(AtomicBool::new(false));

    // let r = running.clone();
    // let s = sleeping.clone();

    // ctrlc::set_handler(move || {
    //     println!("Received Ctrl-C from console.");
    //     r.store(false, Ordering::SeqCst);
    //     if sleeping.load(Ordering::SeqCst) {
    //         println!("Quit from sleeping...");
    //         process::exit(0);
    //     }
    // })
    // .expect("Error setting Ctrl-C handler");

    // let servers = match testconfig.thingworx_servers {
    //     Some(servers) => servers,
    //     None => vec![],
    // };

    // //prepare local disk folder for export.
    // if testconfig.result_export_to_file.enabled {
    //     let path = Path::new(&testconfig.result_export_to_file.folder_name);
    //     if testconfig.result_export_to_file.auto_create_folder {
    //         fs::create_dir_all(&path)?;
    //     }

    //     if !path.exists() {
    //         error!(
    //             "Can't find export folder or can't create export folder:{}",
    //             testconfig.result_export_to_file.folder_name
    //         );
    //         process::exit(1);
    //     }
    // }

    // let path = Path::new(&testconfig.result_export_to_file.folder_name);
    // while running.load(Ordering::SeqCst) {
    //     let now = Instant::now();
    //     let mut total_points: Vec<Point> = Vec::new();

    //     match &testconfig.testmachine.repeat_sampling {
    //         Some(ref x) => {
    //             debug!("start repeated sampling...");
    //             let point = sampling::sampling_repeat(
    //                 &testconfig.testmachine.testid,
    //                 x,
    //                 &path,
    //                 testconfig.result_export_to_file.enabled,
    //             )
    //             .await;
    //             //debug!("sampling_repeat: {:?}\n", point);

    //             match point {
    //                 Ok(p) => total_points.push(p),
    //                 Err(e) => {
    //                     error!("Error:{}", e);
    //                 }
    //             }
    //         }
    //         None => {}
    //     }

    //     // for server in &servers {
    //     //     let points = sampling::sampling_thingworx(
    //     //         server,
    //     //         &path,
    //     //         testconfig.result_export_to_file.enabled,
    //     //         sampling_timeout_inseconds,
    //     //     ).await;
    //     //     debug!("thingworx_servers:{:?}\n", points);
    //     //     match points {
    //     //         Ok(mut ps) => total_points.append(&mut ps),
    //     //         Err(e) => {
    //     //             info!("Error:{}", e);
    //     //         }
    //     //     }
    //     // }
    //     let mut tasks = vec![];

    //     for server in &servers {
    //         let test_server = server.clone();
    //         let enabled = testconfig.result_export_to_file.enabled;
    //         let local_path = testconfig.result_export_to_file.folder_name.clone();
    //         let task = tokio::spawn(async move {
    //             // let points =
    //             sampling::sampling_thingworx(
    //                 &test_server,
    //                 &local_path,
    //                 enabled,
    //                 sampling_timeout_inseconds,
    //             )
    //             .await
    //             // ;
    //             // debug!("thingworx_servers:{:?}", points);
    //             // let points = match points {
    //             //     Ok(points) => points,
    //             //     Err(e) => {
    //             //         error!("{:?}", e);
    //             //         vec![]
    //             //     },
    //             // };
    //             // local_tx.send(points).await
    //         });
    //         tasks.push(task);
    //     }

    //     // while let Some(mut points) = rx.recv().await{
    //     //     debug!("thingworx_servers result received:{}", points.len());
    //     //     total_points.append(&mut points);
    //     // }
    //     for task in tasks {
    //         match task.await {
    //             Err(e) => {
    //                 error!("Error happened in task: {:?}", e);
    //             }
    //             Ok(res) => match res {
    //                 Err(e) => {
    //                     error!("Error happened in task sampling: {:?}", e);
    //                 }
    //                 Ok(mut vec) => total_points.append(&mut vec),
    //             },
    //         }
    //     }

    //     debug!("Total Points:{}", total_points.len());

    //     if testconfig.result_export_to_db.enabled && total_points.len() > 0 {
    //         let myclient = MyInfluxClient::new(&testconfig.result_export_to_db);

    //         match myclient
    //             .write_points(Points::create_new(total_points))
    //             .await
    //         {
    //             Ok(()) => {}
    //             Err(e) => {
    //                 error!("Error: {}", e);
    //             }
    //         }
    //     }

    //     if !running.load(Ordering::SeqCst) {
    //         break;
    //     }

    //     //debug!("Sleeping...");
    //     s.store(true, Ordering::SeqCst);
    //     let new_now = Instant::now();
    //     let delta = match sleep_duration > new_now.duration_since(now) + Duration::from_secs(1) {
    //         true => sleep_duration - new_now.duration_since(now),
    //         false => sleep_duration,
    //     };

    //     info!("Sleeping:{:?}", delta);

    //     delay_for(delta).await;
    //     s.store(false, Ordering::SeqCst);
    // }

    log::info!("we have done.");

    Ok(())
}
