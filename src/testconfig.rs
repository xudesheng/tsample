use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};

// use url::Url;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Owner {
    pub name: String,
    pub email: String,
    pub organization: String,
}
impl Default for Owner {
    fn default() -> Self {
        Owner {
            name: "Desheng Xu".to_string(),
            email: "xudesheng@gmail.com".to_string(),
            organization: "Demotest IO Inc.".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportToFile {
    pub directory: String,
    pub auto_create_folder: bool,
    pub enabled: bool,
}
impl Default for ExportToFile {
    fn default() -> Self {
        ExportToFile {
            directory: String::from("./export"),
            auto_create_folder: true,
            enabled: false,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThingworxMetric {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub split_desc_asprefix: bool,
    pub name: String,
    pub enabled: bool,
    pub options: Option<Vec<String>>,
    pub sanitize: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExportToInfluxDB {
    pub enabled: bool,
    pub server_name: String,
    #[serde(default = "default_thingworx_port")]
    pub port: u16,
    #[serde(
        default = "default_protocol",
        skip_serializing_if = "is_default_protocol"
    )]
    pub protocol: String,
    pub database: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}
fn default_thingworx_port() -> u16 {
    8080
}

fn is_default_protocol(s: &str) -> bool {
    s == "http"
}
impl Default for ExportToInfluxDB {
    fn default() -> Self {
        ExportToInfluxDB {
            enabled: false,
            server_name: String::from("localhost"),
            port: 8086,
            protocol: String::from("http"),
            database: String::from("thingworx"),
            username: None,
            password: None,
        }
    }
}

// #[derive(Serialize, Deserialize, Debug, Clone, Default)]
// pub struct C3P0Metrics {
//     pub names: Vec<String>,
//     pub metrics: Vec<String>,
// }

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct JmxMetric {
    pub name: String,
    pub object_name_pattern: String,
    pub name_label_alternative: Option<String>,
    #[serde(default = "default_metrics")]
    pub metrics: Vec<String>,
}

fn default_metrics() -> Vec<String> {
    vec![]
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ConnectionServers {
    pub names: Vec<String>,
    pub metrics: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubSystem {
    pub name: String,
    #[serde(default = "default_subsystem_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub split_desc_asprefix: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub sanitize: bool,
}

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

fn default_subsystem_enabled() -> bool {
    true
}

impl Default for SubSystem {
    fn default() -> Self {
        SubSystem {
            name: String::from(""),
            enabled: false,
            options: None,
            split_desc_asprefix: false,
            sanitize: false,
        }
    }
}
// impl SubSystem {
//     pub fn new(name: &str) -> Self {
//         SubSystem {
//             name: name.to_string(),
//             enabled: false,
//             options: None,
//             split_desc_asprefix: false,
//             sanitize: false,
//         }
//     }
// }
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArbitraryMetric {
    pub name: String,
    pub url: String,
    #[serde(default = "default_subsystem_enabled")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub split_desc_asprefix: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub sanitize: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThingworxServer {
    pub name: String,
    pub host: String,
    pub port: u16,
    #[serde(
        default = "default_protocol",
        skip_serializing_if = "is_default_protocol"
    )]
    pub protocol: String,
    #[serde(default = "default_application")]
    pub application: String,
    pub app_key: String,
    pub subsystems: Vec<SubSystem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_servers: Option<ConnectionServers>,
    // pub c3p0_metrics: Option<C3P0Metrics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jmx_metrics: Option<Vec<JmxMetric>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arbitrary_metrics: Option<Vec<ArbitraryMetric>>,
}

impl ThingworxServer {
    pub fn get_arbitrary_access_url(&self, url: &str) -> String {
        let splitter = if url.starts_with('/') { "" } else { "/" };
        format!(
            "{}://{}:{}/{}{}{}",
            self.protocol, self.host, self.port, self.application, splitter, url
        )
    }
    pub fn get_cxserver_query_url(&self) -> String {
        format!(
            "{}://{}:{}/{}/ThingTemplates/ConnectionServer/Services/QueryImplementingThingsWithData",
            self.protocol, self.host, self.port, self.application
        )
    }

    pub fn get_query_mbeanstree_url(&self) -> String {
        format!(
            "{}://{}:{}/{}/Things/JMX.LocalServer/Services/QueryMBeansTree",
            self.protocol, self.host, self.port, self.application
        )
    }

    pub fn get_mbean_attributeinfo_url(&self) -> String {
        format!(
            "{}://{}:{}/{}/Things/JMX.LocalServer/Services/GetMBeanAttributesInfo",
            self.protocol, self.host, self.port, self.application
        )
    }

    pub fn get_cxserver_query_service_url(&self, cxserver_name: &str) -> String {
        format!(
            "{}://{}:{}/{}/Things/{}/Services/GetPerformanceMetrics",
            self.protocol, self.host, self.port, self.application, cxserver_name
        )
    }
}

fn default_protocol() -> String {
    String::from("http")
}
fn default_application() -> String {
    String::from("Thingworx")
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TestConfig {
    pub owner: Option<Owner>,
    #[serde(default = "default_query_time_out")]
    pub query_time_out: u64,
    #[serde(default = "default_scrap_interval")]
    pub scrap_interval: u64,
    #[serde(default = "default_refresh_server_interval")]
    pub refresh_server_interval: u64,
    pub thingworx_servers: Vec<ThingworxServer>,
    pub export_to_influxdb: ExportToInfluxDB,
    pub export_to_file: Option<ExportToFile>,
}

fn default_query_time_out() -> u64 {
    20
}

fn default_scrap_interval() -> u64 {
    30
}

fn default_refresh_server_interval() -> u64 {
    300
}
impl TestConfig {
    pub fn load_from_file(file_name: &str) -> Result<Self> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: TestConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
// impl ThingworxMetric {
//     pub fn get_url(&self) -> String {
//         if self.url.is_none() {
//             format!("Subsystems/{}/Services/GetPerformanceMetrics", self.name)
//         } else {
//             self.url.clone().unwrap()
//         }
//     }

//     pub fn set_options(&mut self, options: Vec<String>) {
//         self.options = Some(options);
//     }

//     pub fn new(
//         // url: String,
//         // split_desc_asprefix: bool,
//         name: String,
//         enabled: bool,
//     ) -> ThingworxMetric {
//         ThingworxMetric {
//             url: None,
//             split_desc_asprefix: true,
//             name,
//             enabled,
//             options: None,
//             sanitize: None,
//         }
//     }
//     // pub fn get_sample()->ThingworxMetric{
//     //     ThingworxMetric{
//     //         url:"Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
//     //         split_desc_asprefix: true,
//     //         name: "ValueStream".to_string(),
//     //         enabled: false,
//     //     }
//     // }
//     pub fn get_samples() -> Vec<ThingworxMetric> {
//         let mut valuestream_options: Vec<String> = Vec::new();

//         valuestream_options.push("totalWritesQueued".to_string());
//         valuestream_options.push("totalWritesPerformed".to_string());

//         let mut m1 = ThingworxMetric::new("ValueStreamProcessingSubsystem".to_string(), true);
//         m1.set_options(valuestream_options);

//         let m2 = ThingworxMetric::new("DataTableProcessingSubsystem".to_string(), false);
//         let m3 = ThingworxMetric::new("EventProcessingSubsystem".to_string(), true);
//         let m4 = ThingworxMetric::new("PlatformSubsystem".to_string(), false);

//         let m5 = ThingworxMetric::new("StreamProcessingSubsystem".to_string(), true);
//         let m6 = ThingworxMetric::new("WSCommunicationsSubsystem".to_string(), false);

//         let m7 = ThingworxMetric::new("WSExecutionProcessingSubsystem".to_string(), false);

//         let m8 = ThingworxMetric::new("TunnelSubsystem".to_string(), false);

//         let m9 = ThingworxMetric::new("AlertProcessingSubsystem".to_string(), false);

//         let m10 = ThingworxMetric::new("FederationSubsystem".to_string(), false);

//         [m1, m2, m3, m4, m5, m6, m7, m8, m9, m10].to_vec()
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ThingworxServer {
//     pub alias: Option<String>,
//     pub host: String,
//     pub port: u16,
//     pub protocol: String,
//     pub application: Option<String>,
//     pub app_key: String,
//     pub metrics: Vec<ThingworxMetric>,
// }

// impl ThingworxServer {
//     pub fn new(
//         alias: Option<String>,
//         host: String,
//         port: u16,
//         protocol: String,
//         application: String,
//         app_key: String,
//         metrics: Vec<ThingworxMetric>,
//     ) -> ThingworxServer {
//         ThingworxServer {
//             alias,
//             host,
//             port,
//             protocol,
//             application: Some(application),
//             app_key,
//             metrics,
//         }
//     }

//     pub fn get_url(&self) -> Result<Url, failure::Error> {
//         let mut url = Url::parse("http://127.0.0.1:8080/")?;
//         url.set_scheme(&self.protocol)
//             .map_err(|err| println!("{:?}", err))
//             .ok();
//         url.set_host(Some(&self.host))
//             .map_err(|err| println!("{:?}", err))
//             .ok();
//         url.set_port(Some(self.port))
//             .map_err(|err| println!("{:?}", err))
//             .ok();
//         url.set_path(match &self.application {
//             Some(app) => app,
//             None => "Thingworx",
//         });

//         Ok(url)
//     }

//     pub fn get_metric_url(&self, metric: &ThingworxMetric) -> Result<Url, failure::Error> {
//         let mut url = Url::parse("http://127.0.0.1:8080/")?;
//         url.set_scheme(&self.protocol)
//             .map_err(|err| println!("{:?}", err))
//             .ok();
//         url.set_host(Some(&self.host))
//             .map_err(|err| println!("{:?}", err))
//             .ok();
//         url.set_port(Some(self.port))
//             .map_err(|err| println!("{:?}", err))
//             .ok();

//         let application = match &self.application {
//             Some(app) => app,
//             None => "Thingworx",
//         };

//         let application = match &metric.url {
//             None => format!(
//                 "{}/Subsystems/{}/Services/GetPerformanceMetrics",
//                 application, metric.name
//             ),
//             Some(url) => {
//                 if url.starts_with("/") {
//                     format!("{}{}", application, url)
//                 } else {
//                     format!("{}/{}", application, url)
//                 }
//             }
//         };

//         url.set_path(&application);

//         Ok(url)
//     }

//     pub fn get_sample() -> ThingworxServer {
//         ThingworxServer {
//             alias: Some("platform_1".to_string()),
//             host: "xxx85.desheng.io".to_string(),
//             port: 443,
//             protocol: "https".to_string(),
//             application: Some("Thingworx".to_string()),
//             app_key: "937230ce-780c-4229-b886-2d3d31fc13xx".to_string(),
//             metrics: ThingworxMetric::get_samples(),
//         }
//     }

//     pub fn get_samples() -> Vec<ThingworxServer> {
//         [ThingworxServer::get_sample()].to_vec()
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TestDataDestination {
//     pub using_udp: bool,
//     pub protocol: Option<String>,
//     pub server_name: String,
//     pub port: usize,
//     pub database: String,
//     pub user: Option<String>,
//     pub password: Option<String>,
//     pub enabled: bool,
// }

// impl TestDataDestination {
//     fn get_sample() -> TestDataDestination {
//         TestDataDestination {
//             using_udp: false,
//             protocol: None,
//             server_name: "127.0.0.1".to_string(),
//             port: 8086,
//             database: "thingworx".to_string(),
//             user: None,
//             password: None,
//             enabled: false,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TestDataExportToDisk {
//     pub auto_create_folder: bool,
//     pub one_time_result_file_name: Option<String>,
//     pub repeat_result_file_name: Option<String>,
//     pub folder_name: String,
//     pub enabled: bool,
// }

// impl TestDataExportToDisk {
//     fn get_sample() -> TestDataExportToDisk {
//         TestDataExportToDisk {
//             auto_create_folder: true,
//             one_time_result_file_name: None,
//             repeat_result_file_name: None,
//             folder_name: "./export".to_string(),
//             enabled: true,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct RepeatTest {
//     pub cpu_load_one: bool,
//     pub cpu_load_five: bool,
//     pub cpu_load_fifteen: bool,
//     pub proc_total: bool,
//     pub mem_total: bool,
//     pub mem_free: bool,
//     pub mem_avail: bool,
//     pub mem_buffers: bool,
//     pub mem_cached: bool,
//     pub mem_used: bool,
//     pub swap_total: bool,
//     pub swap_free: bool,
//     pub disk_total: bool,
//     pub disk_free: bool,
// }

// impl RepeatTest {
//     fn get_sample() -> RepeatTest {
//         RepeatTest {
//             cpu_load_one: true,
//             cpu_load_five: true,
//             cpu_load_fifteen: true,
//             proc_total: true,
//             mem_total: true,
//             mem_free: true,
//             mem_avail: true,
//             mem_buffers: true,
//             mem_cached: true,
//             mem_used: true,
//             swap_total: true,
//             swap_free: true,
//             disk_total: true,
//             disk_free: true,
//         }
//     }
// }
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct OneTimeTest {
//     pub os_type: bool,
//     pub os_release: bool,
//     pub cpu_num: bool,
//     pub cpu_speed: bool,
//     pub hostname: bool,
// }

// impl OneTimeTest {
//     fn get_sample() -> OneTimeTest {
//         OneTimeTest {
//             os_type: true,
//             os_release: true,
//             cpu_num: true,
//             cpu_speed: true,
//             hostname: true,
//         }
//     }
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct TestMachine {
//     pub testid: String,
//     pub sampling_cycle_inseconds: Option<u64>,
//     pub sampling_timeout_inseconds: Option<u64>,
//     pub onetime_sampling: Option<OneTimeTest>,
//     pub repeat_sampling: Option<RepeatTest>,
// }

// impl TestMachine {
//     fn get_sample() -> TestMachine {
//         TestMachine {
//             testid: "twx85".to_string(),
//             sampling_cycle_inseconds: Some(30 as u64),
//             sampling_timeout_inseconds: Some(10 as u64),
//             onetime_sampling: Some(OneTimeTest::get_sample()),
//             repeat_sampling: Some(RepeatTest::get_sample()),
//         }
//     }
// }
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Owner {
//     pub name: String,
//     pub email: String,
//     pub organization: Option<String>,
// }

// impl Owner {
//     fn get_sample() -> Owner {
//         Owner {
//             name: "Desheng Xu".to_string(),
//             email: "dxu@ptc.com".to_string(),
//             organization: Some("PTC Inc.".to_string()),
//         }
//     }
// }
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ThingworxTestConfig {
//     pub title: Option<String>,
//     pub owner: Option<Owner>,
//     pub testmachine: TestMachine,
//     pub thingworx_servers: Option<Vec<ThingworxServer>>,
//     pub result_export_to_db: TestDataDestination,
//     pub result_export_to_file: TestDataExportToDisk,
// }

// impl ThingworxTestConfig {
//     fn get_sample() -> ThingworxTestConfig {
//         ThingworxTestConfig {
//             title: Some("this is a demo.".to_string()),
//             owner: Some(Owner::get_sample()),
//             testmachine: TestMachine::get_sample(),
//             thingworx_servers: Some(ThingworxServer::get_samples()),
//             result_export_to_db: TestDataDestination::get_sample(),
//             result_export_to_file: TestDataExportToDisk::get_sample(),
//         }
//     }

//     pub fn export_sample(filename: &str) -> Result<(), Box<dyn Error>> {
//         let testconfig = ThingworxTestConfig::get_sample();
//         let testconfigstr = ser::to_string(&testconfig)?;
//         fs::write(filename, &testconfigstr[..])?;
//         Ok(())
//     }

//     pub fn from_tomefile(filename: &str) -> Result<ThingworxTestConfig, Box<dyn Error>> {
//         debug!("Reading from file:{:?}", filename);
//         let mut file = fs::File::open(filename)?;
//         let mut contents = String::new();
//         file.read_to_string(&mut contents)?;
//         let testconfig = de::from_slice(contents.as_bytes())?;
//         debug!("{:?}", testconfig);
//         Ok(testconfig)
//     }
// }
