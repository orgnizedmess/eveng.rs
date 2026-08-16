//! Clients and models for managing nodes within a lab.

use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(deserialize_with = "number_from_string")]
    pub config: i32,
    pub console: String,
    pub delay: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_list: Option<Vec<serde_json::Value>>,
    pub icon: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub image: String,
    pub left: i32,
    pub name: String,
    #[serde(flatten)]
    pub node_type: NodeType,
    pub status: i32,
    pub template: String,
    pub top: i32,
    pub url: String,
    pub uuid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateNodeRequest {
    pub config: String,
    pub count: i32,
    pub delay: i32,
    pub icon: String,
    pub left: i32,
    pub name: String,
    #[serde(flatten)]
    pub node_type: NodeType,
    pub postfix: i32,
    pub template: String,
    pub top: i32,
}

#[derive(Debug, Serialize)]
pub struct EditNodeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub node_type: Option<NodeType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNodeResponse {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NodeType {
    Iol(IolParams),
    Qemu(QemuParams),
    Dynamips(DynamipsParams),
    Docker(DockerParams),
    Vpcs(VpcsParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QemuParams {
    pub cpu: i32,
    pub cpulimit: Option<i32>,
    pub ethernet: i32,
    pub ram: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_nic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_options: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamipsParams {
    pub idlepc: String,
    pub nvram: i32,
    pub ram: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IolParams {
    pub ethernet: i32,
    pub nvram: i32,
    pub ram: i32,
    pub serial: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerParams {
    pub ethernet: i32,
    pub ram: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VpcsParams {
    pub ethernet: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    pub network_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interfaces {
    pub ethernet: Vec<Interface>,
    pub id: i32,
    pub serial: Vec<Interface>,
    pub sort: String,
}

// this alone doesn't explain what type is for what
pub type EditInterfaceRequest = HashMap<i32, i32>;

pub struct NodesClient {
    client: Client,
    path: String,
}

impl NodesClient {
    pub(crate) fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    /// Lists all nodes.
    pub async fn list(&self) -> Result<HashMap<i32, Node>> {
        Ok(self
            .client
            .get::<WireMap<i32, Node>>(&format!("labs{}/nodes", self.path))
            .await?
            .into_data()?
            .0)
    }

    /// Adds a new node to the lab.
    pub async fn add(&self, params: &CreateNodeRequest) -> Result<NodeClient> {
        let resp: CreateNodeResponse = self
            .client
            .post(&format!("labs{}/nodes", self.path), params)
            .await?
            .into_data()?;
        Ok(NodeClient::new(self.client.clone(), &self.path, resp.id))
    }

    /// Starts all nodes.
    pub async fn start(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/start", self.path))
            .await?;
        Ok(())
    }

    /// Stops all nodes.
    pub async fn stop(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/stop", self.path))
            .await?;
        Ok(())
    }

    /// Wipes the config from all nodes.
    pub async fn wipe(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/wipe", self.path))
            .await?;
        Ok(())
    }

    /// Exports the config of all nodes.
    ///
    /// TODO: only some might be supported, document that after checking
    pub async fn export(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/export", self.path))
            .await?;
        Ok(())
    }
}

pub struct NodeClient {
    client: Client,
    path: String,
    id: i32,
}

impl NodeClient {
    pub(crate) fn new(client: Client, path: &str, id: i32) -> Self {
        Self {
            client,
            path: path.to_string(),
            id,
        }
    }

    /// Gets the node's details.
    pub async fn get(&self) -> Result<Node> {
        self.client
            .get(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?
            .into_data()
    }

    /// Updates the node's details.
    pub async fn edit(&self, params: &EditNodeRequest) -> Result<()> {
        self.client
            .put::<(), EditNodeRequest>(&format!("labs{}/nodes/{}", self.path, self.id), params)
            .await?;
        Ok(())
    }

    /// Deletes the node.
    pub async fn delete(self) -> Result<()> {
        self.client
            .delete::<()>(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Starts the node.
    pub async fn start(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/start", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Stops the node.
    pub async fn stop(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/stop", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Wipes the node's config.
    ///
    /// TODO: Explain what wipe does
    pub async fn wipe(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/wipe", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Exports the node's config.
    ///
    /// TODO: Explain what export does
    pub async fn export(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/export", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Gets a node's interfaces.
    pub async fn interfaces(&self) -> Result<Interfaces> {
        self.client
            .get(&format!("labs{}/nodes/{}/interfaces", self.path, self.id))
            .await?
            .into_data()
    }

    // Would connect_interface be a better name?
    pub async fn edit_interface(&self, params: &EditInterfaceRequest) -> Result<()> {
        self.client
            .put::<(), EditInterfaceRequest>(
                &format!("labs{}/nodes/{}/interfaces", self.path, self.id),
                params,
            )
            .await?;
        Ok(())
    }
}
