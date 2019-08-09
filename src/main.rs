extern crate sys_info;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate log;
extern crate env_logger;

//use env_logger::{Env};
use std::env;

pub mod thingworxtestconfig;
pub mod thingworxjson;
pub mod sampling;
pub mod myinfluxdb;

use thingworxtestconfig::*;
use myinfluxdb::*;
//use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

extern crate clap;
use clap::{Arg, App}; //, SubCommand
use std::process;
use influx_db_client::{Point, Points};
use std::error::Error;
use std::{thread, time};

extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() ->Result<(),Box<dyn Error>>{
    //let env = pretty_env_logger::Env::new().filter("TSAMPLE_LOG");
    //pretty_env_logger::init_custom_env("TSAMPLE_LOG");
    let log_level = match env::var("TSAMPLE_LOG") {
        Ok(level) => level,
        Err(_) => "info".to_string(),
    };

    env_logger::Builder::new().parse_filters(&log_level).init();
    // env_logger::builder()
    //     .format(|buf, record| {
    //         writeln!(buf, "{}: {}", record.level(), record.args())
    //     })
    //     .init();

    info!("TSAMPLE Started.");

    let matches = App::new("Thingworx Sampler")
            .version("0.0.1")
            .author("Deshneg Xu <dxu@ptc.com>")
            .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG_FILE")
                .help("Configuration file name, it should be a TOML file.")
                .takes_value(true)
                )
            .arg(Arg::with_name("export")
                .help("Export sample configuration file.")
                .short("e")
                .long("export")
                .requires("config")
                )
            // .arg(Arg::with_name("logfile")
            //     .help("Specific log file name other than tsample.log in current folder.")
            //     .short("l")
            //     .long("logfile")
            //     .value_name("LOG_FILE")
            //     .takes_value(true))
            .get_matches();
    let config_file = matches.value_of("config").unwrap_or("./config.toml");
    // let log_file = matches.value_of("logfile").unwrap_or("./tsample.log");

    // match init_log(log_file) {
    //     Ok(()) => println!("log started."),
    //     Err(e) => {println!("Error:{}", e);}
    // };

    if matches.is_present("export") {
        match ThingworxTestConfig::export_sample(config_file) {
            Ok(()) => {
                debug!("Sample configuration file has been exported to:{}", config_file);
                process::exit(0);
            },
            Err(e) => {
                // println!("Failed to export sample configuration file to:{}", config_file);
                // println!("Error message:{}.", e);

                error!("Failed to export sample configuration file to:{}", config_file);
                error!("{:?}", e);
                process::exit(1);
            },
        }
    }
    //tsample::ThingworxTestConfig::export_sample(config_file)?;
    let testconfig = match ThingworxTestConfig::from_tomefile(config_file) {
        Ok(conf) => conf,
        Err(e) => {
            error!("Can't parse configuration file:{},{}", config_file,e);
            process::exit(1);
        }
    };

    let sleep = match testconfig.testmachine.sampling_cycle_inminute {
        Some(minutes) => minutes*60*1000,
        None => 1*60*1000,
    };
    let sleep_duration = time::Duration::from_millis(sleep);

    let running = Arc::new(AtomicBool::new(true));
    let sleeping = Arc::new(AtomicBool::new(false));

    let r = running.clone();
    let s = sleeping.clone();

    ctrlc::set_handler(move || {
        println!("Received Ctrl-C from console.");
        r.store(false, Ordering::SeqCst);
        if sleeping.load(Ordering::SeqCst) {
            println!("Quit from sleeping...");
            process::exit(0);
        }
    }).expect("Error setting Ctrl-C handler");

    let mut has_servers = true;
    let servers = match testconfig.thingworx_servers {
        Some(servers) => servers,
        None => {has_servers=false; vec![]},
    };

    while running.load(Ordering::SeqCst){
        info!("start repeated sampling...");
        let point = sampling::sampling_repeat(&testconfig.testmachine.testid, &testconfig.testmachine.repeat_sampling);
        //debug!("sampling_repeat: {:?}\n", point);

        let mut total_points:Vec<Point> = Vec::new();
        match point {
            Ok(p) => total_points.push(p),
            Err(e) =>{error!("Error:{}", e);},
        }

        if has_servers {
            for server in &servers {
                let points = sampling::sampling_thingworx(server);
                //debug!("thingworx_servers:{:?}\n", points);
                match points {
                    Ok(mut ps) => total_points.append(&mut ps),
                    Err(e) => {info!("Error:{}", e);},
                }
            }
        }
        
        debug!("Total Points:{}", total_points.len());

        let myclient = MyInfluxClient::new(&testconfig.test_data_target);

        match myclient.write_points(Points::create_new(total_points)) {
            Ok(()) => {},
            Err(e) => {error!("Error: {}", e);},
        }

        if !running.load(Ordering::SeqCst){break;}

        info!("Sleeping...");
        s.store(true, Ordering::SeqCst);
        thread::sleep(sleep_duration);
        s.store(false, Ordering::SeqCst);
    }
    
    info!("we have done.");
    

    Ok(())
}
