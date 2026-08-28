use crate::nodes::NodeType;
use crate::utils::WireMap;
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTemplate {
    #[serde(skip)]
    pub name: String,
    pub description: String,
    pub options: HashMap<String, TemplateOption>,
    #[serde(rename = "type")]
    pub node_type: NodeType,
}

impl NodeTemplate {
    pub fn default_map(&self) -> Map<String, Value> {
        let options = &self.options;

        options
            .into_iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect()
    }
}

impl PartialEq<NodeTemplate> for NodeType {
    fn eq(&self, other: &NodeTemplate) -> bool {
        *self == other.node_type
    }
}

impl PartialEq<NodeType> for NodeTemplate {
    fn eq(&self, other: &NodeType) -> bool {
        self.node_type == *other
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOption {
    pub list: Option<Value>,
    pub name: String,
    #[serde(rename = "type")]
    pub option_type: String,
    pub value: Value,
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
