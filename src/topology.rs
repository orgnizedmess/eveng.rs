use crate::utils::{map_or_empty_seq, nested_map_or_empty_seq};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TopologyEntry {
    pub destination: String,
    pub destination_label: String,
    pub destination_type: String,
    pub source: String,
    pub source_label: String,
    pub source_type: String,
    #[serde(rename = "type")]
    pub topology_type: String,
}

pub type Topology = Vec<TopologyEntry>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    #[serde(deserialize_with = "map_or_empty_seq")]
    pub ethernet: HashMap<i32, String>,
    #[serde(deserialize_with = "nested_map_or_empty_seq")]
    pub serial: HashMap<i32, HashMap<i32, String>>,
}

impl Client {
    pub async fn topology(&self) -> Result<Topology> {
        self.get(&format!("/labs/{}/topology", self.lab_path))
            .await?
            .into_data()
    }
    // Part of labs API in source code
    pub async fn links(&self) -> Result<Links> {
        self.get(&format!("/labs/{}/links", self.lab_path))
            .await?
            .into_data()
    }
}
