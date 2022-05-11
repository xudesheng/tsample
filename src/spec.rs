// use std::fmt::{Display, Formatter};

use influxdb::Timestamp;
use influxdb::Type as SpecType;

#[derive(Debug, Clone)]
pub struct WriteSpec {
    pub fields: Vec<(String, SpecType)>,
    pub tags: Vec<(String, SpecType)>,
    pub measurement: String,
    pub timestamp: Timestamp,
}

impl WriteSpec {
    pub fn new<S>(timestamp: Timestamp, measurement: S) -> Self
    where
        S: Into<String>,
    {
        WriteSpec {
            fields: vec![],
            tags: vec![],
            measurement: measurement.into(),
            timestamp,
        }
    }

    pub fn add_field<S>(mut self, field: S, value: SpecType) -> Self
    where
        S: Into<String>,
    {
        self.fields.push((field.into(), value));
        self
    }

    pub fn add_tag<S>(mut self, tag: S, value: SpecType) -> Self
    where
        S: Into<String>,
    {
        self.tags.push((tag.into(), value));
        self
    }

    // pub fn get_precision(&self) -> String {
    //     let modifier = match self.timestamp {
    //         Timestamp::Nanoseconds(_) => "ns",
    //         Timestamp::Microseconds(_) => "u",
    //         Timestamp::Milliseconds(_) => "ms",
    //         Timestamp::Seconds(_) => "s",
    //         Timestamp::Minutes(_) => "m",
    //         Timestamp::Hours(_) => "h",
    //     };
    //     modifier.to_string()
    // }
}
