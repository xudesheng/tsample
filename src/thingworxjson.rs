use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;

#[derive(Serialize, Deserialize, Debug)]
pub struct TwxJson {
    #[serde(skip_deserializing)]
    pub data_shape: Map<String, String>,
    pub rows: Vec<RowData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RowData {
    pub description: String,
    pub name: String,
    pub value: f64,
}
