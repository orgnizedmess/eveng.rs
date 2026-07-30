use crate::{Client, Result};
use crate::utils::empty_vec_as_map;
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
    #[serde(deserialize_with = "empty_vec_as_map")]
    pub ethernet: HashMap<String, String>,
    #[serde(deserialize_with = "empty_vec_as_map")]
    pub serial: HashMap<String, HashMap<String, String>>,
}

impl Client {
    pub async fn topology(&self) -> Result<Topology> {
        self.get(&format!("labs/{}/topology", self.lab_path))
            .await?
            .into_data()
    }
    // Part of labs API in source code
    pub async fn links(&self) -> Result<Links> {
        self.get(&format!("labs/{}/links", self.lab_path))
            .await?
            .into_data()
    }
}
