//! Types and clients to manage node templates in an EVE-NG instance.

use crate::nodes::NodeType;
use crate::utils::{WireMap, map_or_seq, number_from_string};
use crate::{Client, Result};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// A client to manage node templates.
pub struct TemplatesClient {
    client: Client,
}

impl TemplatesClient {
    pub(crate) fn new(client: Client) -> Self {
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

/// Type to describe a node template.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTemplate {
    /// Name of the template.
    #[serde(skip)]
    pub name: String,
    /// Description of the template.
    pub description: String,
    /// List of options for the template.
    pub options: TemplateOptions,
    /// Type of node the template creates.
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

/// Type to describe the list of options in a [`NodeTemplate`].
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateOptions {
    /// Startup config status of the node.
    pub config: ListOption<u8>,
    /// Seconds to wait before starting the node.
    pub delay: InputOption<u32>,
    /// Icon used to display the node in the lab.
    pub icon: ListOption<String>,
    /// Name used to display the node in the lab.
    pub name: InputOption<String>,
    /// Number of configured CPUs on the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<InputOption<u32>>,
    /// CPU limit status of the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpulimit: Option<CheckboxOption>,
    /// Number of configured Ethernet interfaces/portgroups (Docker, Dynamips,
    /// IOL and QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<InputOption<u32>>,
    /// Idle PC for the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idlepc: Option<InputOption<String>>,
    /// Image used to boot the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ListOption<String>>,
    /// NVRAM configured on the node, in kilobytes (IOL and Dynamips).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram: Option<InputOption<u32>>,
    /// QEMU version used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_version: Option<ListOption>,
    /// QEMU target architecture used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_arch: Option<ListOption>,
    /// QEMU NIC model used for the node's interfaces (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_nic: Option<ListOption>,
    /// Custom options passed to QEMU when creating the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_options: Option<InputOption<String>>,
    /// RAM configured on the node, in megabytes (Docker, Dynamips, IOL, QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<InputOption<u32>>,
    /// Number of configured Serial portgroups (IOL only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<InputOption<u32>>,
    /// Module configured in slot 1 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<ListOption>,
    /// Module configured in slot 2 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<ListOption>,
    /// Module configured in slot 3 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot3: Option<ListOption>,
    /// Module configured in slot 4 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot4: Option<ListOption>,
    /// Module configured in slot 5 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot5: Option<ListOption>,
    /// Module configured in slot 6 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot6: Option<ListOption>,
    /// UUID configured for the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<InputOption<String>>,
}

/// Type to describe a template option containing a list of possible values
/// a user could choose from.
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "
    K: Eq + std::hash::Hash + std::str::FromStr + serde::Deserialize<'de>,
    K::Err: std::fmt::Display
"))]
pub struct ListOption<K = String> {
    /// Name of the option.
    pub name: String,
    /// List of possible values.
    #[serde(deserialize_with = "map_or_seq")]
    pub list: HashMap<K, String>,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    #[serde(deserialize_with = "number_from_string")]
    pub value: K,
}

/// Type to describe a template option that takes text input from the user.
#[derive(Debug, Serialize, Deserialize)]
pub struct InputOption<T> {
    /// Name of the option.
    pub name: String,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    pub value: T,
}

/// Type to describe a template option that takes a boolean value.
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckboxOption {
    /// Name of the option.
    pub name: String,
    /// Type of the option.
    #[serde(rename = "type")]
    pub option_type: String,
    /// Default value of the option.
    pub value: u8,
}

/// A client to manage a single node template.
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
