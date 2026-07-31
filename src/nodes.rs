use crate::{Client, Result};
use crate::utils::number_from_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Node {
    #[serde(deserialize_with = "number_from_string")]
    pub config: i32,
    pub delay: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_list: Option<Vec<serde_json::Value>>,
    pub icon: String,
    pub id: Option<i32>,
    pub left: i32,
    pub name: String,
    pub status: i32,
    pub template: String,
    pub top: i32,
    #[serde(rename = "type")]
    pub node_type: String,
    pub url: String,
    #[serde(flatten)]
    pub kind: NodeKind,
}

pub type Nodes = HashMap<String, Node>;

#[derive(Debug, Serialize)]
pub struct CreateNodeRequest {
    pub config: String,
    pub count: i32,
    pub delay: i32,
    pub icon: String,
    pub left: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub postfix: i32,
    pub template: String,
    pub top: i32,
    #[serde(flatten)]
    pub kind: NodeKind,
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
    pub kind: Option<NodeKind>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNodeResponse {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeKind {
    Iol(IolParams),
    Qemu(QemuParams),
    Dynamips(DynamipsParams),
    Docker(DockerParams),
    Vpcs(VpcsParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QemuParams {
    console: String,
    cpu: String,
    cpulimit: String,
    ethernet: String,
    image: String,
    ram: String,
    qemu_version: String,
    qemu_arch: String,
    qemu_nic: String,
    qemu_options: String,
    uuid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamipsParams {
    idlepc: String,
    image: String,
    nvram: String,
    ram: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IolParams {
    ethernet: String,
    image: String,
    nvram: String,
    ram: String,
    serial: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerParams {
    ethernet: String,
    image: String,
    ram: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VpcsParams {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interface {
    name: String,
    network_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interfaces {
    ethernet: Vec<Interface>,
    id: i32,
    serial: Vec<Interface>,
    sort: String,
}

// this alone doesn't explain what type is for what
pub type EditInterfaceRequest = HashMap<i32, i32>;

#[derive(Serialize, Deserialize)]
pub struct Template {
    description: String,
    options: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    qemu: Option<serde_json::Value>,
    #[serde(rename = "type")]
    template_type: String,
}

impl Client {
    pub async fn nodes(&self) -> Result<Nodes> {
        self.get(&format!("labs/{}/nodes", self.lab_path))
            .await?
            .into_data()
    }

    pub async fn node(&self, id: i32) -> Result<Node> {
        self.get(&format!("labs/{}/nodes/{}", self.lab_path, id))
            .await?
            .into_data()
    }

    pub async fn add_node(&self, params: &CreateNodeRequest) -> Result<CreateNodeResponse> {
        self.post::<CreateNodeResponse, CreateNodeRequest>(
            &format!("labs/{}/nodes", self.lab_path),
            params,
        )
        .await?
        .into_data()
    }

    pub async fn edit_node(&self, id: i32, params: &EditNodeRequest) -> Result<()> {
        self.put::<(), EditNodeRequest>(&format!("labs/{}/nodes/{}", self.lab_path, id), params)
            .await?;
        Ok(())
    }

    pub async fn start_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/start", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn start_node(&self, id: &str) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/start/{}", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn wipe_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/wipe", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn wipe_node(&self, id: &str) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/wipe/{}", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn export_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/export", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn export_node(&self, id: &str) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/export/{}", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn delete_node(&self, id: i32) -> Result<()> {
        self.delete::<()>(&format!("labs/{}/nodes/{}", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn interfaces(&self, node_id: i32) -> Result<Interfaces> {
        self.get(&format!("labs/{}/nodes/{}/interfaces", self.lab_path, node_id))
            .await?
            .into_data()
    }

    // Would connect_interface be a better name?
    pub async fn edit_interface(
        &self,
        node_id: i32,
        params: &EditInterfaceRequest,
    ) -> Result<()> {
        self.put::<(), EditInterfaceRequest>(&format!("labs/{}/nodes/{}/interfaces", self.lab_path, node_id), params)
            .await?;
        Ok(())
    }

    pub async fn node_templates(&self) -> Result<serde_json::Value> {
        self.get("list/templates/").await?.into_data()
    }

    pub async fn node_template(&self, template: &str) -> Result<Template> {
        self.get(&format!("list/templates/{}", template))
            .await?
            .into_data()
    }
}
