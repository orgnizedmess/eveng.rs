use crate::nodes::NodeType;
use crate::utils::{WireMap, map_or_seq, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTemplate {
    #[serde(skip)]
    pub name: String,
    pub description: String,
    pub options: TemplateOptions,
    #[serde(rename = "type")]
    pub node_type: NodeType,
}

impl NodeTemplate {
    pub(crate) fn defaults_map(&self) -> Result<Map<String, Value>> {
        let defaults = serde_json::to_value(&self.options)?;
        let map = defaults.as_object().unwrap();

        Ok(map
            .iter()
            .map(|(k, v)| (k.clone(), v.get("value").cloned().unwrap_or_default()))
            .collect())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOptions {
    pub config: ListOption<u8>,

    pub delay: InputOption<u32>,

    pub icon: ListOption<String>,

    pub name: InputOption<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<InputOption<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpulimit: Option<CheckboxOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<InputOption<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ListOption<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<InputOption<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_version: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_arch: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_nic: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_options: Option<InputOption<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<InputOption<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub idlepc: Option<InputOption<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram: Option<InputOption<u32>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot3: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot4: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot5: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot6: Option<ListOption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<InputOption<u32>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "
    K: Eq + std::hash::Hash + std::str::FromStr + serde::Deserialize<'de>,
    K::Err: std::fmt::Display
"))]
pub struct ListOption<K = String> {
    pub name: String,
    #[serde(deserialize_with = "map_or_seq")]
    pub list: HashMap<K, String>,
    #[serde(rename = "type")]
    pub option_type: String,
    #[serde(deserialize_with = "number_from_string")]
    pub value: K,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputOption<T> {
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: String,
    pub value: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckboxOption {
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: String,
    pub value: u8,
}

pub struct TemplatesClient {
    client: Client,
}

impl TemplatesClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Lists available node templates.
    pub async fn list(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/templates/")
            .await?
            .into_data()?
            .0)
    }
}

pub struct TemplateClient {
    client: Client,
    name: String,
}

impl TemplateClient {
    pub(crate) fn new(client: Client, name: impl Into<String>) -> Self {
        Self {
            client,
            name: name.into(),
        }
    }

    /// Gets the template's details.
    pub async fn get(&self) -> Result<NodeTemplate> {
        let mut resp: NodeTemplate = self
            .client
            .get(&format!("list/templates/{}", self.name))
            .await?
            .into_data()?;
        resp.name = self.name.clone();

        Ok(resp)
    }
}
