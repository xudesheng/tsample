use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;
use serde_json::Value as JsonValue;

#[derive(Serialize, Deserialize, Debug)]
pub struct TwxJson {
    #[serde(skip_deserializing)]
    pub data_shape: Map<String, String>,
    pub rows: Vec<RowData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RowData {
    pub description: Option<String>,
    pub name: String,
    pub value: Option<JsonValue>,
}
