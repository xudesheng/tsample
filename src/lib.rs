extern crate serde;

use serde::{Serialize, Deserialize};
use toml::ser;
use toml::de;
use std::fs;
use std::error::Error;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ThingworxMetric{
    url: String,
    split_desc_asprefix : bool,
    name: String,
    enabled: bool,
}

impl ThingworxMetric{
    pub fn new(url:String, split_desc_asprefix: bool, name: String, enabled: bool)->ThingworxMetric{
        ThingworxMetric{
            url,
            split_desc_asprefix,
            name,
            enabled,
        }
    }
    pub fn get_sample()->ThingworxMetric{
        ThingworxMetric{
            url:"Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "ValueStream".to_string(),
            enabled: false,
        }
    }
    pub fn get_samples()->Vec<ThingworxMetric>{
        
        let m1 = ThingworxMetric{
            url:"Subsystems/ValueStreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "ValueStreamProcessingSubsystem".to_string(),
            enabled: true,
        };
        let m2 = ThingworxMetric{
            url:"Subsystems/DataTableProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix: true,
            name: "DataTableProcessingSubsystem".to_string(),
            enabled: false,
        };
        let m3 = ThingworxMetric{
            url:"Subsystems/EventProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : false,
            name: "EventProcessingSubsystem".to_string(),
            enabled: true,
        };
        let m4 = ThingworxMetric{
            url:"Subsystems/PlatformSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "PlatformSubsystem".to_string(),
            enabled: false,
        };

        let m5 = ThingworxMetric{
            url:"Subsystems/StreamProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name : "StreamProcessingSubsystem".to_string(),
            enabled: true,
        };
        let m6 = ThingworxMetric{
            url:"Subsystems/WSCommunicationsSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name : "WSCommunicationsSubsystem".to_string(),
            enabled: false,
        };

        let m7 = ThingworxMetric{
            url:"Subsystems/WSExecutionProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "WSExecutionProcessingSubsystem".to_string(),
            enabled: false,
        };

        let m8 = ThingworxMetric{
            url:"Subsystems/TunnelSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "TunnelSubsystem".to_string(),
            enabled: false,
        };

        let m9 = ThingworxMetric{
            url:"Subsystems/AlertProcessingSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "AlertProcessingSubsystem".to_string(),
            enabled:false,
        };

        let m10 = ThingworxMetric{
            url:"Subsystems/FederationSubsystem/Services/GetPerformanceMetrics".to_string(),
            split_desc_asprefix : true,
            name: "FederationSubsystem".to_string(),
            enabled: false,
        };

        [m1,m2,m3,m4,m5,m6,m7,m8,m9,m10].to_vec()
    }
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct TestServerConfig {
    host: String,
    port: usize,
    protocol: String,
    application: Option<String>,
    app_key: String,
    metric: Vec<ThingworxMetric>,
}

impl TestServerConfig{
    pub fn new(host:String, port: usize, protocol:String, 
        application:String, app_key: String, 
        metric:Vec<ThingworxMetric>)->TestServerConfig{
            TestServerConfig{
                host,
                port,
                protocol,
                application:Some(application),
                app_key,
                metric,
            }
    }

    pub fn get_sample() -> TestServerConfig{
        TestServerConfig{
            host: "twx85.desheng.io".to_string(),
            port: 433,
            protocol: "https".to_string(),
            application: Some("Thingworx".to_string()),
            app_key: "937230ce-780c-4229-b886-2d3d31fc1302".to_string(),
            metric: ThingworxMetric::get_samples(),
        }
    }

    pub fn get_samples() -> Vec<TestServerConfig>{
        [TestServerConfig::get_sample()].to_vec()
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct TargetServer {
    alias: String,
    server_config: TestServerConfig,
}

impl TargetServer {
    fn get_sample() -> TargetServer {
        TargetServer{
            alias: "platform_1".to_string(),
            server_config: TestServerConfig::get_sample(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct TestDataDestination{
    communication_type : String,
    server_name: String,
    port: usize,
    database: String,
    user: String,
    password: String,
}

impl TestDataDestination{
    fn get_sample() ->TestDataDestination{
        TestDataDestination{
            communication_type:"UDP".to_string(),
            server_name:"10.100.0.13".to_string(),
            port: 8089,
            database: "thingworx".to_string(),
            user: "twadmin".to_string(),
            password: "twadmin".to_string(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct RepeatTest{
    mem_info_one: bool,
    mem_info_five: bool,
    mem_info_fifteen: bool,
    proc_total: bool,
    mem_total: bool,
    mem_free: bool,
    mem_avail: bool,
    mem_buffers: bool,
    mem_cached: bool,
    swap_total: bool,
    swap_free: bool,
    disk_total: bool,
    disk_free: bool,
}

impl RepeatTest{
    fn get_sample() ->RepeatTest{
        RepeatTest{
            mem_info_one: false,
            mem_info_five: false,
            mem_info_fifteen: false,
            proc_total: false,
            mem_total: false,
            mem_free: false,
            mem_avail: false,
            mem_buffers: false,
            mem_cached: false,
            swap_total: false,
            swap_free: false,
            disk_total: false,
            disk_free: false,
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct OneTimeTest {
    os_type: bool,
    os_release: bool,
    cpu_num: bool,
    cpu_speed: bool,
    hostname: bool,
}

impl OneTimeTest {
    fn get_sample() ->OneTimeTest{
        OneTimeTest{
            os_type: false,
            os_release: false,
            cpu_num: false,
            cpu_speed: false,
            hostname: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct TestMachine {
    testid: String,
    onetime_sampling:OneTimeTest,
    repeat_sampling:RepeatTest,
}

impl TestMachine{
    fn get_sample() ->TestMachine{
        TestMachine{
            testid: "twx85".to_string(),
            onetime_sampling: OneTimeTest::get_sample(),
            repeat_sampling: RepeatTest::get_sample(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Owner{
    name: String,
    email: String,
    organization: Option<String>,
}

impl Owner{
    fn get_sample() ->Owner{
        Owner{
            name: "Desheng Xu".to_string(),
            email: "dxu@ptc.com".to_string(),
            organization: Some("PTC Inc.".to_string()),
        }
    }
    
}
#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct ThingworxTestConfig{
    title: Option<String>,
    owner: Option<Owner>,
    testmachine: TestMachine,
    thingworx_servers:Vec<TestServerConfig>,
    test_data_target: TestDataDestination,
}

impl ThingworxTestConfig{
    fn get_sample() ->ThingworxTestConfig{
        ThingworxTestConfig{
            title:Some("this is a demo.".to_string()),
            owner: Some(Owner::get_sample()),
            testmachine: TestMachine::get_sample(),
            thingworx_servers: TestServerConfig::get_samples(),
            test_data_target: TestDataDestination::get_sample(),
        }
    }

    pub fn export_sample(filename: &str) -> Result<(), Box<dyn Error>>{
        let testconfig = ThingworxTestConfig::get_sample();
        let testconfigstr = ser::to_string(&testconfig)?;
        fs::write(filename, &testconfigstr[..])?;
        Ok(())
    }

    pub fn from_tomefile(filename: &str) -> Result<ThingworxTestConfig, Box<dyn Error>>{
        let mut file = fs::File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let testconfig = de::from_slice(contents.as_bytes())?;
        Ok(testconfig)
    }
}