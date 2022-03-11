use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap as Map;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionServerResults {
    #[serde(skip_deserializing)]
    pub data_shape: Map<String, String>,
    pub rows: Vec<ConnectionServerRow>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionServerRow {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryMBeansTree {
    #[serde(skip_deserializing)]
    pub data_shape: Map<String, String>,
    pub rows: Vec<QueryMBeansTreeRow>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryMBeansTreeRow {
    pub node_name: String,
    pub object_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MBeansAttributeInfo {
    #[serde(skip_deserializing)]
    pub data_shape: Map<String, String>,
    pub rows: Vec<MBeansAttributeInfoRow>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MBeansAttributeInfoRow {
    pub name: String,
    pub object_name: String,
    pub preview: String,
    pub type_: String,
}
