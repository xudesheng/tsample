use crate::thingworxtestconfig::TestDataDestination;
///InfluxDB container.
/// docker run -p 8086:8086 -p 8089:8089/udp -e INFLUXDB_UDP_ENABLED=true -v /Users/desheng/docker/influxdb/data/:/var/lib/influxdb influxdb:1.7.3
/// docker run -p 8086:8086 -p 8089:8089/udp -e INFLUXDB_UDP_ENABLED=true -v /Users/desheng/docker/influxdb/data/:/var/lib/influxdb --name=influxdb influxdb
///
/// Creating a DB named mydb:
/// $ curl -G http://localhost:8086/query --data-urlencode "q=CREATE DATABASE thingworx"
/// Inserting into the DB:
/// $ curl -i -XPOST 'http://localhost:8086/write?db=thingworx' --data-binary 'cpu_load_short,host=server01,region=us-west value=0.64 1434055562000000000'
///
/// client from another VM: docker run --rm --link=influxdb -it influxdb influx -host influxdb
///
/// https://github.com/driftluo/InfluxDBClient-rs
///
//extern crate influx_db_client;
use influx_db_client::{Client, Points, Precision};
use std::error::Error;

pub struct MyInfluxClient {
    pub is_udp: bool,
    url: String,
    user: Option<String>,
    password: Option<String>,
    database: String,
}

impl MyInfluxClient {
    pub fn new(data_target: &TestDataDestination) -> MyInfluxClient {
        //unimplemented!()

        let url = match data_target.using_udp {
            true => format!("{}:{}", data_target.server_name, data_target.port),
            false => {
                let protocol = match &data_target.protocol {
                    None => "http",
                    Some(p) => &p,
                };
                format!(
                    "{}://{}:{}",
                    protocol, data_target.server_name, data_target.port
                )
            }
        };

        MyInfluxClient {
            is_udp: data_target.using_udp,
            url: url.to_string(),
            user: match &data_target.user {
                None => None,
                Some(user) => Some(user.to_string()),
            },
            password: match &data_target.password {
                None => None,
                Some(pass) => Some(pass.to_string()),
            },
            database: (&data_target.database).to_string(),
        }
    }

    pub fn write_points(&self, points: Points) -> Result<(), Box<dyn Error>> {
        // let client = Client::new(&self.url, &self.database);
        let client = match (&self.user, &self.password) {
            (Some(user), Some(pass)) => {
                Client::new(&self.url, &self.database).set_authentication(user, pass)
            }
            (_, _) => Client::new(&self.url, &self.database),
        };

        // let client = &mut client;
        let _ = client.write_points(points, Some(Precision::Milliseconds), None)?;
        Ok(())
    }
}
