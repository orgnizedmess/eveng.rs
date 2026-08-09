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

pub type Nodes = HashMap<i32, Node>;

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

#[derive(Serialize, Deserialize)]
pub struct Template {
    description: String,
    options: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    qemu: Option<serde_json::Value>,
    #[serde(rename = "type")]
    template_type: String,
}

pub type Templates = HashMap<String, String>;

impl Client {
    pub async fn nodes(&self) -> Result<Nodes> {
        Ok(self
            .get::<WireMap<i32, Node>>(&format!("labs/{}/nodes", self.lab_path))
            .await?
            .into_data()?
            .0)
    }

    pub async fn node(&self, id: i32) -> Result<Node> {
        self.get(&format!("labs/{}/nodes/{}", self.lab_path, id))
            .await?
            .into_data()
    }

    pub async fn add_node(&self, params: CreateNodeRequest) -> Result<CreateNodeResponse> {
        self.post::<CreateNodeResponse, CreateNodeRequest>(
            &format!("labs/{}/nodes", self.lab_path),
            params,
        )
        .await?
        .into_data()
    }

    pub async fn edit_node(&self, id: i32, params: EditNodeRequest) -> Result<()> {
        self.put::<(), EditNodeRequest>(&format!("labs/{}/nodes/{}", self.lab_path, id), params)
            .await?;
        Ok(())
    }

    pub async fn start_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/start", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn start_node(&self, id: i32) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/{}/start", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn stop_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/stop", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn stop_node(&self, id: i32) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/{}/stop", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn wipe_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/wipe", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn wipe_node(&self, id: i32) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/{}/wipe", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn export_nodes(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/export", self.lab_path))
            .await?;
        Ok(())
    }

    pub async fn export_node(&self, id: i32) -> Result<()> {
        self.get::<()>(&format!("labs/{}/nodes/{}/export", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn delete_node(&self, id: i32) -> Result<()> {
        self.delete::<()>(&format!("labs/{}/nodes/{}", self.lab_path, id))
            .await?;
        Ok(())
    }

    pub async fn interfaces(&self, node_id: i32) -> Result<Interfaces> {
        self.get(&format!(
            "labs/{}/nodes/{}/interfaces",
            self.lab_path, node_id
        ))
        .await?
        .into_data()
    }

    // Would connect_interface be a better name?
    pub async fn edit_interface(&self, node_id: i32, params: EditInterfaceRequest) -> Result<()> {
        self.put::<(), EditInterfaceRequest>(
            &format!("labs/{}/nodes/{}/interfaces", self.lab_path, node_id),
            params,
        )
        .await?;
        Ok(())
    }

    pub async fn node_templates(&self) -> Result<Templates> {
        Ok(self
            .get::<WireMap<String, String>>("list/templates/")
            .await?
            .into_data()?
            .0)
    }

    pub async fn node_template(&self, template: &str) -> Result<Template> {
        self.get(&format!("list/templates/{}", template))
            .await?
            .into_data()
    }
}
