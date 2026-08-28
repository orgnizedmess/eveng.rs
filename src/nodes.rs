//! Clients and models for managing nodes within a lab.

use crate::interfaces::InterfaceType;
use crate::interfaces::{InterfaceClient, InterfacesClient};
use crate::templates::NodeTemplate;
use crate::utils::{WireMap, empty_string_is_none, number_from_string};
use crate::{Client, Error, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(deserialize_with = "number_from_string")]
    pub config: u32,

    pub delay: u32,

    pub icon: String,

    pub left: u32,

    pub name: String,

    #[serde(rename = "type")]
    pub node_type: NodeType,

    pub status: u32,

    pub template: String,

    pub top: u32,

    pub url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_list: Option<Vec<Value>>,

    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub console: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpulimit: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<u32>,

    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub image: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<u32>,

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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub idlepc: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Qemu,
    Iol,
    Docker,
    Dynamips,
    Vpcs,
}

impl PartialEq<Node> for NodeType {
    fn eq(&self, other: &Node) -> bool {
        *self == other.node_type
    }
}

impl PartialEq<NodeType> for Node {
    fn eq(&self, other: &NodeType) -> bool {
        self.node_type == *other
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = match self {
            NodeType::Qemu => "qemu",
            NodeType::Iol => "iol",
            NodeType::Docker => "docker",
            NodeType::Dynamips => "dynamips",
            NodeType::Vpcs => "vpcs",
        };

        f.write_str(value)
    }
}

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
    pub async fn list(&self) -> Result<HashMap<u32, Node>> {
        Ok(self
            .client
            .get::<WireMap<u32, Node>>(&format!("labs{}/nodes", self.path))
            .await?
            .into_data()?
            .0)
    }

    pub async fn add<T: TypedNode>(&self, params: AddNodeRequest<T>) -> Result<NodeClient> {
        #[derive(Deserialize)]
        struct CreateNodeResponse {
            id: u32,
        }

        let resp: CreateNodeResponse = self
            .client
            .post(&format!("labs{}/nodes", self.path), &params)
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
    id: u32,
}

impl NodeClient {
    pub(crate) fn new(client: Client, path: &str, id: u32) -> Self {
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
    pub async fn edit<T: TypedNode>(&self, params: EditNodeRequest<T>) -> Result<()> {
        self.client
            .put::<(), EditNodeRequest<T>>(&format!("labs{}/nodes/{}", self.path, self.id), &params)
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

    pub fn interfaces(&self) -> InterfacesClient {
        InterfacesClient::new(self.client.clone(), &self.path, self.id)
    }

    pub fn ethernet(&self, id: u32) -> InterfaceClient {
        InterfaceClient::new(
            self.client.clone(),
            &self.path,
            self.id,
            id,
            InterfaceType::Ethernet,
        )
    }

    pub fn serial(&self, id: u32) -> InterfaceClient {
        InterfaceClient::new(
            self.client.clone(),
            &self.path,
            self.id,
            id,
            InterfaceType::Serial,
        )
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct QemuParams {
    cpu: u32,
    cpulimit: u32,
    ethernet: u32,
    image: String,
    ram: u32,
    qemu_version: String,
    qemu_arch: String,
    qemu_nic: String,
    qemu_options: String,
    uuid: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DynamipsParams {
    idlepc: String,
    image: String,
    nvram: u32,
    ram: u32,
    slot1: String,
    slot2: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IolParams {
    ethernet: u32,
    image: String,
    nvram: u32,
    ram: u32,
    serial: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DockerParams {
    ethernet: u32,
    image: String,
    ram: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct VpcsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    ethernet: Option<u32>,
}

mod private {
    pub trait Sealed {}
}

pub trait TypedNode: Serialize + DeserializeOwned + private::Sealed {
    const NODE_TYPE: NodeType;
}

impl private::Sealed for QemuParams {}
impl TypedNode for QemuParams {
    const NODE_TYPE: NodeType = NodeType::Qemu;
}

impl private::Sealed for DockerParams {}
impl TypedNode for DockerParams {
    const NODE_TYPE: NodeType = NodeType::Docker;
}

impl private::Sealed for DynamipsParams {}
impl TypedNode for DynamipsParams {
    const NODE_TYPE: NodeType = NodeType::Dynamips;
}

impl private::Sealed for VpcsParams {}
impl TypedNode for VpcsParams {
    const NODE_TYPE: NodeType = NodeType::Vpcs;
}

impl private::Sealed for IolParams {}
impl TypedNode for IolParams {
    const NODE_TYPE: NodeType = NodeType::Iol;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddNodeRequest<T> {
    left: u32,
    count: u32,
    #[serde(rename = "type")]
    node_type: NodeType,
    template: String,
    top: u32,
    config: String,
    delay: u32,
    icon: String,
    name: String,

    #[serde(flatten)]
    params: T,
}

impl<T: TypedNode> AddNodeRequest<T> {
    pub fn from_template(template: &NodeTemplate) -> Result<Self> {
        if template.node_type != T::NODE_TYPE {
            return Err(Error::Invalid(format!(
                "incorrect node type: expected {}, got {}",
                template.node_type,
                T::NODE_TYPE
            )));
        }

        if template.description.ends_with(".missing") {
            return Err(Error::Invalid("image missing for template".into()));
        }

        let mut defaults = template.default_map();
        defaults.insert("type".to_string(), serde_json::json!(template.node_type));
        defaults.insert("template".to_string(), serde_json::json!(template.name));
        defaults.insert("left".to_string(), serde_json::json!(0));
        defaults.insert("top".to_string(), serde_json::json!(0));
        defaults.insert("count".to_string(), serde_json::json!(1));

        serde_json::from_value(Value::Object(defaults)).map_err(Into::into)
    }

    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn config(mut self, config: impl Into<String>) -> Self {
        self.config = config.into();
        self
    }

    pub fn delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }
}

impl AddNodeRequest<QemuParams> {
    pub fn qemu(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    pub fn cpu(mut self, cpu: u32) -> Self {
        self.params.cpu = cpu;
        self
    }

    pub fn cpulimit(mut self, cpulimit: u32) -> Self {
        self.params.cpulimit = cpulimit;
        self
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn qemu_version(mut self, qemu_version: impl Into<String>) -> Self {
        self.params.qemu_version = qemu_version.into();
        self
    }

    pub fn qemu_arch(mut self, qemu_arch: impl Into<String>) -> Self {
        self.params.qemu_arch = qemu_arch.into();
        self
    }

    pub fn qemu_nic(mut self, qemu_nic: impl Into<String>) -> Self {
        self.params.qemu_nic = qemu_nic.into();
        self
    }

    pub fn qemu_options(mut self, qemu_options: impl Into<String>) -> Self {
        self.params.qemu_options = qemu_options.into();
        self
    }

    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.params.uuid = uuid.into();
        self
    }
}

impl AddNodeRequest<DynamipsParams> {
    pub fn dynamips(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    pub fn idlepc(mut self, idlepc: impl Into<String>) -> Self {
        self.params.idlepc = idlepc.into();
        self
    }

    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn slot1(mut self, slot1: impl Into<String>) -> Self {
        self.params.slot1 = slot1.into();
        self
    }

    pub fn slot2(mut self, slot2: impl Into<String>) -> Self {
        self.params.slot2 = slot2.into();
        self
    }
}

impl AddNodeRequest<IolParams> {
    pub fn iol(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn serial(mut self, serial: u32) -> Self {
        self.params.serial = serial;
        self
    }
}

impl AddNodeRequest<DockerParams> {
    pub fn docker(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }
}

impl AddNodeRequest<VpcsParams> {
    pub fn vpcs(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditNodeRequest<T> {
    left: u32,
    top: u32,
    config: u32,
    delay: u32,
    icon: String,
    name: String,
    #[serde(rename = "type")]
    node_type: NodeType,
    template: String,

    #[serde(flatten)]
    params: T,
}

impl<T: TypedNode> EditNodeRequest<T> {
    fn from_node(node: &Node) -> Result<Self> {
        if node.node_type != T::NODE_TYPE {
            return Err(Error::Invalid(format!(
                "incorrect node type: expected {}, got {}",
                node.node_type,
                T::NODE_TYPE
            )));
        }

        let value = serde_json::to_value(node)?;
        serde_json::from_value(value).map_err(Into::into)
    }

    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn config(mut self, config: u32) -> Self {
        self.config = config.into();
        self
    }

    pub fn delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }
}

impl EditNodeRequest<QemuParams> {
    pub fn qemu(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    pub fn cpu(mut self, cpu: u32) -> Self {
        self.params.cpu = cpu;
        self
    }

    pub fn cpulimit(mut self, cpulimit: u32) -> Self {
        self.params.cpulimit = cpulimit;
        self
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn qemu_version(mut self, qemu_version: impl Into<String>) -> Self {
        self.params.qemu_version = qemu_version.into();
        self
    }

    pub fn qemu_arch(mut self, qemu_arch: impl Into<String>) -> Self {
        self.params.qemu_arch = qemu_arch.into();
        self
    }

    pub fn qemu_nic(mut self, qemu_nic: impl Into<String>) -> Self {
        self.params.qemu_nic = qemu_nic.into();
        self
    }

    pub fn qemu_options(mut self, qemu_options: impl Into<String>) -> Self {
        self.params.qemu_options = qemu_options.into();
        self
    }

    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.params.uuid = uuid.into();
        self
    }
}

impl EditNodeRequest<DynamipsParams> {
    pub fn dynamips(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    pub fn idlepc(mut self, idlepc: impl Into<String>) -> Self {
        self.params.idlepc = idlepc.into();
        self
    }

    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn slot1(mut self, slot1: impl Into<String>) -> Self {
        self.params.slot1 = slot1.into();
        self
    }

    pub fn slot2(mut self, slot2: impl Into<String>) -> Self {
        self.params.slot2 = slot2.into();
        self
    }
}

impl EditNodeRequest<IolParams> {
    pub fn iol(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    pub fn serial(mut self, serial: u32) -> Self {
        self.params.serial = serial;
        self
    }
}

impl EditNodeRequest<DockerParams> {
    pub fn docker(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }
}

impl EditNodeRequest<VpcsParams> {
    pub fn vpcs(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }
}
