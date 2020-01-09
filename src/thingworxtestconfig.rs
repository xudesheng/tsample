extern crate serde;
extern crate url;

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::Read;
use toml::de;
use toml::ser;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThingworxMetric {
    pub url: String,
    pub split_desc_asprefix: bool,
    pub name: String,
    pub enabled: bool,
    pub options: Option<Vec<String>>,
}

impl ThingworxMetric {
    pub fn new(
        url: String,
        split_desc_asprefix: bool,
        name: String,
        enabled: bool,
    ) -> ThingworxMetric {
        ThingworxMetric {
            url,
            split_desc_asprefix,
            name,
            enabled,
            options: None,
        }
    }
    // pub fn get_sample()->ThingworxMetric{
    //     ThingworxMetric{
    //         url:"Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
    //         split_desc_asprefix: true,
    //         name: "ValueStream".to_string(),
    //         enabled: false,
    //     }
    // }
    pub fn get_samples() -> Vec<ThingworxMetric> {
        let mut valuestream_options: Vec<String> = Vec::new();

        valuestream_options.push("totalWritesQueued".to_string());
        valuestream_options.push("totalWritesPerformed".to_string());

        let m1 = ThingworxMetric {
            url: "Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics"
                .to_string(),
            split_desc_asprefix: true,
            name: "ValueStreamProcessingSubsystem".to_string(),
            enabled: true,
            options: Some(valuestream_options),
        };
        let m2 = ThingworxMetric {
            url: "Subsystems/DataTableProcessingSubsystem/Services/GetPerformanceMetrics"
                .to_string(),
            split_desc_asprefix: true,
            name: "DataTableProcessingSubsystem".to_string(),
            enabled: false,
            options: None,
        };
        let m3 = ThingworxMetric {
            url: "Subsystems/EventProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: false,
            name: "EventProcessingSubsystem".to_string(),
            enabled: true,
            options: None,
        };
        let m4 = ThingworxMetric {
            url: "Subsystems/PlatformSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "PlatformSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        let m5 = ThingworxMetric {
            url: "Subsystems/StreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "StreamProcessingSubsystem".to_string(),
            enabled: true,
            options: None,
        };
        let m6 = ThingworxMetric {
            url: "Subsystems/WSCommunicationsSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "WSCommunicationsSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        let m7 = ThingworxMetric {
            url: "Subsystems/WSExecutionProcessingSubsystem/Services/GetPerformanceMetrics"
                .to_string(),
            split_desc_asprefix: true,
            name: "WSExecutionProcessingSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        let m8 = ThingworxMetric {
            url: "Subsystems/TunnelSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "TunnelSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        let m9 = ThingworxMetric {
            url: "Subsystems/AlertProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "AlertProcessingSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        let m10 = ThingworxMetric {
            url: "Subsystems/FederationSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "FederationSubsystem".to_string(),
            enabled: false,
            options: None,
        };

        [m1, m2, m3, m4, m5, m6, m7, m8, m9, m10].to_vec()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThingworxServer {
    pub alias: Option<String>,
    pub host: String,
    pub port: u16,
    pub protocol: String,
    pub application: Option<String>,
    pub app_key: String,
    pub metrics: Vec<ThingworxMetric>,
}

impl ThingworxServer {
    pub fn new(
        alias: Option<String>,
        host: String,
        port: u16,
        protocol: String,
        application: String,
        app_key: String,
        metrics: Vec<ThingworxMetric>,
    ) -> ThingworxServer {
        ThingworxServer {
            alias,
            host,
            port,
            protocol,
            application: Some(application),
            app_key,
            metrics,
        }
    }

    pub fn get_url(&self) -> Result<Url, Box<dyn Error>> {
        let mut url = Url::parse("http://127.0.0.1:8080/")?;
        url.set_scheme(&self.protocol)
            .map_err(|err| println!("{:?}", err))
            .ok();
        url.set_host(Some(&self.host))
            .map_err(|err| println!("{:?}", err))
            .ok();
        url.set_port(Some(self.port))
            .map_err(|err| println!("{:?}", err))
            .ok();
        url.set_path(match &self.application {
            Some(app) => app,
            None => "Thingworx",
        });

        Ok(url)
    }

    pub fn get_sample() -> ThingworxServer {
        ThingworxServer {
            alias: Some("platform_1".to_string()),
            host: "xxx85.desheng.io".to_string(),
            port: 443,
            protocol: "https".to_string(),
            application: Some("Thingworx".to_string()),
            app_key: "937230ce-780c-4229-b886-2d3d31fc13xx".to_string(),
            metrics: ThingworxMetric::get_samples(),
        }
    }

    pub fn get_samples() -> Vec<ThingworxServer> {
        [ThingworxServer::get_sample()].to_vec()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestDataDestination {
    pub using_udp: bool,
    pub protocol: Option<String>,
    pub server_name: String,
    pub port: usize,
    pub database: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
}

impl TestDataDestination {
    fn get_sample() -> TestDataDestination {
        TestDataDestination {
            using_udp: false,
            protocol: None,
            server_name: "127.0.0.1".to_string(),
            port: 8086,
            database: "thingworx".to_string(),
            user: None,
            password: None,
            enabled: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestDataExportToDisk {
    pub auto_create_folder: bool,
    pub one_time_result_file_name: Option<String>,
    pub repeat_result_file_name: Option<String>,
    pub folder_name: String,
    pub enabled: bool,
}

impl TestDataExportToDisk {
    fn get_sample() -> TestDataExportToDisk {
        TestDataExportToDisk {
            auto_create_folder: true,
            one_time_result_file_name: None,
            repeat_result_file_name: None,
            folder_name: "./export".to_string(),
            enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepeatTest {
    pub cpu_load_one: bool,
    pub cpu_load_five: bool,
    pub cpu_load_fifteen: bool,
    pub proc_total: bool,
    pub mem_total: bool,
    pub mem_free: bool,
    pub mem_avail: bool,
    pub mem_buffers: bool,
    pub mem_cached: bool,
    pub swap_total: bool,
    pub swap_free: bool,
    pub disk_total: bool,
    pub disk_free: bool,
}

impl RepeatTest {
    fn get_sample() -> RepeatTest {
        RepeatTest {
            cpu_load_one: true,
            cpu_load_five: true,
            cpu_load_fifteen: true,
            proc_total: true,
            mem_total: true,
            mem_free: true,
            mem_avail: true,
            mem_buffers: true,
            mem_cached: true,
            swap_total: true,
            swap_free: true,
            disk_total: true,
            disk_free: true,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OneTimeTest {
    pub os_type: bool,
    pub os_release: bool,
    pub cpu_num: bool,
    pub cpu_speed: bool,
    pub hostname: bool,
}

impl OneTimeTest {
    fn get_sample() -> OneTimeTest {
        OneTimeTest {
            os_type: true,
            os_release: true,
            cpu_num: true,
            cpu_speed: true,
            hostname: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestMachine {
    pub testid: String,
    pub sampling_cycle_inseconds: Option<u64>,
    pub onetime_sampling: Option<OneTimeTest>,
    pub repeat_sampling: Option<RepeatTest>,
}

impl TestMachine {
    fn get_sample() -> TestMachine {
        TestMachine {
            testid: "twx85".to_string(),
            sampling_cycle_inseconds: Some(120 as u64),
            onetime_sampling: Some(OneTimeTest::get_sample()),
            repeat_sampling: Some(RepeatTest::get_sample()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    pub name: String,
    pub email: String,
    pub organization: Option<String>,
}

impl Owner {
    fn get_sample() -> Owner {
        Owner {
            name: "Desheng Xu".to_string(),
            email: "dxu@ptc.com".to_string(),
            organization: Some("PTC Inc.".to_string()),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThingworxTestConfig {
    pub title: Option<String>,
    pub owner: Option<Owner>,
    pub testmachine: TestMachine,
    pub thingworx_servers: Option<Vec<ThingworxServer>>,
    pub result_export_to_db: TestDataDestination,
    pub result_export_to_file: TestDataExportToDisk,
}

impl ThingworxTestConfig {
    fn get_sample() -> ThingworxTestConfig {
        ThingworxTestConfig {
            title: Some("this is a demo.".to_string()),
            owner: Some(Owner::get_sample()),
            testmachine: TestMachine::get_sample(),
            thingworx_servers: Some(ThingworxServer::get_samples()),
            result_export_to_db: TestDataDestination::get_sample(),
            result_export_to_file: TestDataExportToDisk::get_sample(),
        }
    }

    pub fn export_sample(filename: &str) -> Result<(), Box<dyn Error>> {
        let testconfig = ThingworxTestConfig::get_sample();
        let testconfigstr = ser::to_string(&testconfig)?;
        fs::write(filename, &testconfigstr[..])?;
        Ok(())
    }

    pub fn from_tomefile(filename: &str) -> Result<ThingworxTestConfig, Box<dyn Error>> {
        debug!("Reading from file:{:?}", filename);
        let mut file = fs::File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let testconfig = de::from_slice(contents.as_bytes())?;
        debug!("{:?}", testconfig);
        Ok(testconfig)
    }
}
