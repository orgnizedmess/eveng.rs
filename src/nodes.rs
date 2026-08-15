use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct NodeInfo {
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

pub struct Nodes {
    client: Client,
    path: String,
}

impl Nodes {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    pub async fn list(&self) -> Result<HashMap<i32, NodeInfo>> {
        Ok(self
            .client
            .get::<WireMap<i32, NodeInfo>>(&format!("labs{}/nodes", self.path))
            .await?
            .into_data()?
            .0)
    }

    pub async fn add(&self, params: &CreateNodeRequest) -> Result<Node> {
        let resp: CreateNodeResponse = self
            .client
            .post(&format!("labs{}/nodes", self.path), params)
            .await?
            .into_data()?;
        Ok(Node::new(self.client.clone(), &self.path, resp.id))
    }

    pub async fn start(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/start", self.path))
            .await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/stop", self.path))
            .await?;
        Ok(())
    }

    pub async fn wipe(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/wipe", self.path))
            .await?;
        Ok(())
    }

    pub async fn export(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/export", self.path))
            .await?;
        Ok(())
    }
}

pub struct Node {
    client: Client,
    path: String,
    id: i32,
}

impl Node {
    pub fn new(client: Client, path: &str, id: i32) -> Self {
        Self {
            client,
            path: path.to_string(),
            id,
        }
    }

    pub async fn get(&self) -> Result<NodeInfo> {
        self.client
            .get(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?
            .into_data()
    }

    pub async fn edit(&self, params: &EditNodeRequest) -> Result<()> {
        self.client
            .put::<(), EditNodeRequest>(&format!("labs{}/nodes/{}", self.path, self.id), params)
            .await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        self.client
            .delete::<()>(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?;
        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/start", self.path, self.id))
            .await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/stop", self.path, self.id))
            .await?;
        Ok(())
    }

    pub async fn wipe(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/wipe", self.path, self.id))
            .await?;
        Ok(())
    }

    pub async fn export(&self) -> Result<()> {
        self.client
            .get::<()>(&format!("labs{}/nodes/{}/export", self.path, self.id))
            .await?;
        Ok(())
    }

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
