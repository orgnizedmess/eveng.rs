//! Types and clients for managing nodes within a lab.

use crate::interfaces::{Ethernet, InterfaceClient, InterfacesClient, Serial};
use crate::labs::{LabClient, LabPath};
use crate::templates::NodeTemplate;
use crate::utils::{WireMap, empty_string_is_none, private::Sealed};
use crate::{Client, Error, Result};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::time::Duration;

/// Type to describe a node in a lab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    /// Startup config status of the node.
    pub config: StartupConfig,
    /// Seconds to wait before starting the node.
    pub delay: u32,
    /// Icon used to display the node in the lab.
    pub icon: String,
    /// Image used to boot the node.
    pub image: String,
    /// Left margin of the node.
    pub left: u32,
    /// Name used to display the node in the lab.
    pub name: String,
    /// Type of the node.
    #[serde(rename = "type")]
    pub node_type: NodeType,
    /// Run status of the node.
    pub status: NodeStatus,
    /// Template used to provide default values for the node.
    pub template: String,
    /// Top margin of the node.
    pub top: u32,
    /// URL to the console session of the node.
    pub url: String,
    /// List of available startup config templates for the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_list: Option<Vec<Value>>,
    /// Type of console of the node.
    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub console: Option<String>,
    /// ID of the node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    /// Number of configured CPUs on the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<u32>,
    /// CPU limit status of the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpulimit: Option<u8>,
    /// Number of configured Ethernet interfaces/portgroups (Docker, Dynamips,
    /// IOL and QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethernet: Option<u32>,
    /// Idle PC for the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idlepc: Option<String>,
    /// NVRAM configured on the node, in kilobytes (IOL and Dynamips).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nvram: Option<u32>,
    /// QEMU version used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_version: Option<String>,
    /// QEMU target architecture used to boot the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_arch: Option<String>,
    /// QEMU NIC model used for the node's interfaces (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_nic: Option<String>,
    /// Custom options passed to QEMU when creating the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qemu_options: Option<String>,
    /// RAM configured on the node, in megabytes (Docker, Dynamips, IOL, QEMU).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ram: Option<u32>,
    /// Number of configured Serial portgroups (IOL only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<u32>,
    /// Module configured in slot 1 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot1: Option<String>,
    /// Module configured in slot 2 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot2: Option<String>,
    /// Module configured in slot 3 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot3: Option<String>,
    /// Module configured in slot 4 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot4: Option<String>,
    /// Module configured in slot 5 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot5: Option<String>,
    /// Module configured in slot 6 of the node (Dynamips only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot6: Option<String>,
    /// UUID configured for the node (QEMU only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
}

/// Type of the node.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    Docker,
    Dynamips,
    Iol,
    Qemu,
    Vpcs,
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

/// Run status of the node.
#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NodeStatus {
    Stopped = 0,
    Starting = 1,
    Running = 2,
    Stopping = 3,
}

/// Startup config status of the node.
#[derive(Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum StartupConfig {
    None = 0,
    Exported = 1,
}

impl From<u8> for StartupConfig {
    fn from(v: u8) -> Self {
        match v {
            0 => StartupConfig::None,
            1 => StartupConfig::Exported,
            _ => StartupConfig::None,
        }
    }
}

/// A client to manage nodes.
pub struct NodesClient {
    client: Client,
    path: LabPath,
}

impl NodesClient {
    pub(crate) fn new(client: Client, path: LabPath) -> Self {
        Self { client, path }
    }

    fn lab(&self) -> LabClient {
        LabClient::from_path(self.client.clone(), self.path.clone())
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

    /// Adds a node.
    pub async fn add<T: TypedNode>(&self, params: AddNodeRequest<T>) -> Result<NodeClient> {
        self.lab().open().await?;

        #[derive(Deserialize)]
        struct CreateNodeResponse {
            id: u32,
        }

        let resp: CreateNodeResponse = self
            .client
            .post(&format!("labs{}/nodes", self.path), &params)
            .await?
            .into_data()?;

        Ok(NodeClient::new(
            self.client.clone(),
            self.path.clone(),
            resp.id,
        ))
    }

    /// Starts all nodes.
    pub async fn start(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/start", self.path))
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// Stops all nodes.
    pub async fn stop(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/stop", self.path))
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// Wipes the existing config of all nodes.
    pub async fn wipe(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/wipe", self.path))
            .await?;
        Ok(())
    }

    /// Exports the running config of all supported nodes as startup configs.
    pub async fn export(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/export", self.path))
            .await?;
        Ok(())
    }
}

/// A client to manage a single node.
pub struct NodeClient {
    client: Client,
    path: LabPath,
    id: u32,
}

impl NodeClient {
    pub(crate) fn new(client: Client, path: LabPath, id: u32) -> Self {
        Self { client, path, id }
    }

    fn lab(&self) -> LabClient {
        LabClient::from_path(self.client.clone(), self.path.clone())
    }

    /// Gets the details of the node.
    pub async fn get(&self) -> Result<Node> {
        self.client
            .get(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?
            .into_data()
    }

    /// Gets the run status of the node.
    pub async fn status(&self) -> Result<NodeStatus> {
        Ok(self.get().await?.status)
    }

    /// Gets the type of the node.
    pub async fn node_type(&self) -> Result<NodeType> {
        Ok(self.get().await?.node_type)
    }

    /// Updates the details of the node.
    pub async fn edit<T: TypedNode>(&self, params: EditNodeRequest<T>) -> Result<()> {
        self.lab().open().await?;

        if matches!(params.status, NodeStatus::Running) {
            return Err(Error::Client(
                "Cannot edit node as it is still running.".to_string(),
            ));
        }

        self.client
            .put::<(), EditNodeRequest<T>>(&format!("labs{}/nodes/{}", self.path, self.id), &params)
            .await?;

        Ok(())
    }

    /// Deletes the node.
    pub async fn delete(self) -> Result<()> {
        self.lab().open().await?;

        if matches!(self.status().await?, NodeStatus::Running) {
            return Err(Error::Client(
                "Cannot delete node as it is still running.".to_string(),
            ));
        }

        self.client
            .delete::<()>(&format!("labs{}/nodes/{}", self.path, self.id))
            .await?;

        Ok(())
    }

    /// Starts the node.
    pub async fn start(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/{}/start", self.path, self.id))
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// Stops the node.
    pub async fn stop(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/{}/stop", self.path, self.id))
            .await?;

        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    /// Wipes the existing config of the node.
    pub async fn wipe(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/{}/wipe", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Exports the existing config of the node as a startup config. Returns
    /// an error if export is unsupported for the node.
    ///
    /// The node's existing config needs to be [wiped] before booting from the
    /// exported config.
    ///
    /// [wiped]: Self::wipe
    pub async fn export(&self) -> Result<()> {
        self.lab().open().await?;

        self.client
            .get::<()>(&format!("labs{}/nodes/{}/export", self.path, self.id))
            .await?;
        Ok(())
    }

    /// Returns a client to manage interfaces.
    pub fn interfaces(&self) -> InterfacesClient {
        InterfacesClient::new(self.client.clone(), self.path.clone(), self.id)
    }

    /// Returns a client to manage an ethernet interface.
    pub fn ethernet(&self, id: u32) -> InterfaceClient<Ethernet> {
        InterfaceClient::ethernet(self.client.clone(), self.path.clone(), self.id, id)
    }

    /// Returns a client to manage a serial interface.
    pub fn serial(&self, id: u32) -> InterfaceClient<Serial> {
        InterfaceClient::serial(self.client.clone(), self.path.clone(), self.id, id)
    }
}

/// Parameters specific to a node of type [`NodeType::Docker`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Docker {
    ethernet: u32,
    image: String,
    ram: u32,
}

/// Parameters specific to a node of type [`NodeType::Dynamips`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Dynamips {
    idlepc: String,
    image: String,
    nvram: u32,
    ram: u32,
    slot1: String,
    slot2: String,
    slot3: Option<String>,
    slot4: Option<String>,
    slot5: Option<String>,
    slot6: Option<String>,
}

/// Parameters specific to a node of type [`NodeType::Iol`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Iol {
    ethernet: u32,
    image: String,
    nvram: u32,
    ram: u32,
    serial: u32,
}

/// Parameters specific to a node of type [`NodeType::Qemu`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Qemu {
    cpu: u32,
    cpulimit: u8,
    ethernet: u32,
    image: String,
    ram: u32,
    qemu_version: String,
    qemu_arch: String,
    qemu_nic: String,
    qemu_options: String,
    uuid: String,
}

/// Parameters specific to a node of type [`NodeType::Vpcs`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vpcs {
    #[serde(skip_serializing_if = "Option::is_none")]
    ethernet: Option<u32>,
}

/// Trait that binds static node types to [`NodeType`].
pub trait TypedNode: Default + Serialize + DeserializeOwned + Sealed {
    /// Type of the node.
    const NODE_TYPE: NodeType;
}

impl Sealed for Qemu {}
impl TypedNode for Qemu {
    const NODE_TYPE: NodeType = NodeType::Qemu;
}

impl Sealed for Docker {}
impl TypedNode for Docker {
    const NODE_TYPE: NodeType = NodeType::Docker;
}

impl Sealed for Dynamips {}
impl TypedNode for Dynamips {
    const NODE_TYPE: NodeType = NodeType::Dynamips;
}

impl Sealed for Vpcs {}
impl TypedNode for Vpcs {
    const NODE_TYPE: NodeType = NodeType::Vpcs;
}

impl Sealed for Iol {}
impl TypedNode for Iol {
    const NODE_TYPE: NodeType = NodeType::Iol;
}

/// Request to add a node.
#[derive(Debug, Serialize, Deserialize)]
pub struct AddNodeRequest<T> {
    count: u32,
    left: u32,
    #[serde(rename = "type")]
    node_type: NodeType,
    template: String,
    top: u32,
    config: StartupConfig,
    delay: u32,
    icon: String,
    name: String,

    #[serde(flatten, default)]
    params: T,
}

impl<T: TypedNode> AddNodeRequest<T> {
    fn from_template(template: &NodeTemplate) -> Result<Self> {
        if template.node_type != T::NODE_TYPE {
            return Err(Error::Client(format!(
                "Incorrect type, expected '{}', got '{}'",
                template.node_type,
                T::NODE_TYPE
            )));
        }

        if template.description.ends_with(".missing") {
            return Err(Error::Client(format!(
                "Cannot create node as image for template '{}' is missing",
                template.name
            )));
        }

        let mut defaults = template.defaults_map()?;
        defaults.insert("type".to_string(), serde_json::json!(template.node_type));
        defaults.insert("template".to_string(), serde_json::json!(template.name));
        defaults.insert("left".to_string(), serde_json::json!(0));
        defaults.insert("top".to_string(), serde_json::json!(0));
        defaults.insert("count".to_string(), serde_json::json!(1));

        serde_json::from_value(Value::Object(defaults)).map_err(Into::into)
    }

    /// Startup config status of the node.
    pub fn config(mut self, config: StartupConfig) -> Self {
        self.config = config;
        self
    }

    /// Seconds to wait before starting the node.
    pub fn delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }

    /// Icon used to display the node in the lab.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Name used to display the node in the lab.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Position of the node in the lab.
    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left;
        self.top = top;
        self
    }
}

impl AddNodeRequest<Docker> {
    /// Creates a request to add a node of type [`NodeType::Docker`].
    pub fn docker(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    /// Number of configured Ethernet interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }
}

impl AddNodeRequest<Dynamips> {
    /// Creates a request to add a node of type [`NodeType::Dynamips`].
    pub fn dynamips(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    /// Idle PC for the node.
    pub fn idlepc(mut self, idlepc: impl Into<String>) -> Self {
        self.params.idlepc = idlepc.into();
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// NVRAM configured on the node, in kilobytes.
    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// Module configured in slot 1 of the node.
    pub fn slot1(mut self, slot1: impl Into<String>) -> Self {
        self.params.slot1 = slot1.into();
        self
    }

    /// Module configured in slot 2 of the node.
    pub fn slot2(mut self, slot2: impl Into<String>) -> Self {
        self.params.slot2 = slot2.into();
        self
    }

    /// Module configured in slot 3 of the node.
    pub fn slot3(mut self, slot3: impl Into<String>) -> Self {
        self.params.slot3 = Some(slot3.into());
        self
    }

    /// Module configured in slot 4 of the node.
    pub fn slot4(mut self, slot4: impl Into<String>) -> Self {
        self.params.slot4 = Some(slot4.into());
        self
    }

    /// Module configured in slot 5 of the node.
    pub fn slot5(mut self, slot5: impl Into<String>) -> Self {
        self.params.slot5 = Some(slot5.into());
        self
    }

    /// Module configured in slot 6 of the node.
    pub fn slot6(mut self, slot6: impl Into<String>) -> Self {
        self.params.slot6 = Some(slot6.into());
        self
    }
}

impl AddNodeRequest<Iol> {
    /// Creates a request to add a node of type [`NodeType::Iol`].
    pub fn iol(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    /// Number of configured Ethernet portgroups.
    /// A portgroup configures 4 interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// NVRAM configured on the node, in kilobytes.
    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// Number of configured Serial portgroups.
    /// A portgroup configures 4 interfaces.
    pub fn serial(mut self, serial: u32) -> Self {
        self.params.serial = serial;
        self
    }
}

impl AddNodeRequest<Qemu> {
    /// Creates a request to add a node of type [`NodeType::Qemu`].
    pub fn qemu(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }

    /// Number of configured CPUs on the node.
    pub fn cpu(mut self, cpu: u32) -> Self {
        self.params.cpu = cpu;
        self
    }

    /// Flag to indicate CPU limit status.
    pub fn cpulimit(mut self, cpulimit: u8) -> Self {
        self.params.cpulimit = cpulimit;
        self
    }

    /// Number of configured Ethernet interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// QEMU version used to boot the node.
    pub fn qemu_version(mut self, qemu_version: impl Into<String>) -> Self {
        self.params.qemu_version = qemu_version.into();
        self
    }

    /// QEMU target architecture used to boot the node.
    pub fn qemu_arch(mut self, qemu_arch: impl Into<String>) -> Self {
        self.params.qemu_arch = qemu_arch.into();
        self
    }

    /// QEMU NIC model used for the interfaces of the node.
    pub fn qemu_nic(mut self, qemu_nic: impl Into<String>) -> Self {
        self.params.qemu_nic = qemu_nic.into();
        self
    }

    /// Custom options passed to QEMU to boot the node.
    pub fn qemu_options(mut self, qemu_options: impl Into<String>) -> Self {
        self.params.qemu_options = qemu_options.into();
        self
    }

    /// UUID configured for the node.
    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.params.uuid = uuid.into();
        self
    }
}

impl AddNodeRequest<Vpcs> {
    /// Creates a request to add a node of type [`NodeType::Vpcs`].
    pub fn vpcs(template: &NodeTemplate) -> Result<Self> {
        Self::from_template(template)
    }
}

/// Creates a request to edit a node.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditNodeRequest<T> {
    left: u32,
    top: u32,
    config: StartupConfig,
    delay: u32,
    icon: String,
    name: String,
    #[serde(rename = "type")]
    node_type: NodeType,
    #[serde(skip_serializing)]
    status: NodeStatus,
    template: String,

    #[serde(flatten, default)]
    params: T,
}

impl<T: TypedNode> EditNodeRequest<T> {
    pub(crate) fn from_node(node: &Node) -> Result<Self> {
        if node.node_type != T::NODE_TYPE {
            return Err(Error::Client(format!(
                "Incorrect type, expected '{}', got '{}'",
                node.node_type,
                T::NODE_TYPE
            )));
        }

        let value = serde_json::to_value(node)?;
        serde_json::from_value(value).map_err(Into::into)
    }

    /// Startup config status of the node.
    pub fn config(mut self, config: StartupConfig) -> Self {
        self.config = config;
        self
    }

    /// Seconds to wait before starting the node.
    pub fn delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }

    /// Icon used to display the node in the lab.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = icon.into();
        self
    }

    /// Name used to display the node in the lab.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Position of the node in the lab.
    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left;
        self.top = top;
        self
    }
}

impl EditNodeRequest<Docker> {
    /// Creates a request to edit a node of type [`NodeType::Docker`].
    pub fn docker(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    /// Number of configured Ethernet interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }
}

impl EditNodeRequest<Dynamips> {
    /// Creates a request to edit a node of type [`NodeType::Dynamips`].
    pub fn dynamips(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    /// Idle PC for the node.
    pub fn idlepc(mut self, idlepc: impl Into<String>) -> Self {
        self.params.idlepc = idlepc.into();
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// NVRAM configured on the node, in kilobytes.
    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// Module configured in slot 1 of the node.
    pub fn slot1(mut self, slot1: impl Into<String>) -> Self {
        self.params.slot1 = slot1.into();
        self
    }

    /// Module configured in slot 2 of the node.
    pub fn slot2(mut self, slot2: impl Into<String>) -> Self {
        self.params.slot2 = slot2.into();
        self
    }

    /// Module configured in slot 3 of the node.
    pub fn slot3(mut self, slot3: impl Into<String>) -> Self {
        self.params.slot3 = Some(slot3.into());
        self
    }

    /// Module configured in slot 4 of the node.
    pub fn slot4(mut self, slot4: impl Into<String>) -> Self {
        self.params.slot4 = Some(slot4.into());
        self
    }

    /// Module configured in slot 5 of the node.
    pub fn slot5(mut self, slot5: impl Into<String>) -> Self {
        self.params.slot5 = Some(slot5.into());
        self
    }

    /// Module configured in slot 6 of the node.
    pub fn slot6(mut self, slot6: impl Into<String>) -> Self {
        self.params.slot6 = Some(slot6.into());
        self
    }
}

impl EditNodeRequest<Iol> {
    /// Creates a request to edit a node of type [`NodeType::Iol`].
    pub fn iol(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    /// Number of configured Ethernet portgroups.
    /// A portgroup configures 4 interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// NVRAM configured on the node, in kilobytes.
    pub fn nvram(mut self, nvram: u32) -> Self {
        self.params.nvram = nvram;
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// Number of configured Serial portgroups.
    /// A portgroup configures 4 interfaces.
    pub fn serial(mut self, serial: u32) -> Self {
        self.params.serial = serial;
        self
    }
}

impl EditNodeRequest<Qemu> {
    /// Creates a request to edit a node of type [`NodeType::Qemu`].
    pub fn qemu(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }

    /// Number of configured CPUs on the node.
    pub fn cpu(mut self, cpu: u32) -> Self {
        self.params.cpu = cpu;
        self
    }

    /// Flag to indicate CPU limit status.
    pub fn cpulimit(mut self, cpulimit: u8) -> Self {
        self.params.cpulimit = cpulimit;
        self
    }

    /// Number of configured Ethernet interfaces.
    pub fn ethernet(mut self, ethernet: u32) -> Self {
        self.params.ethernet = ethernet;
        self
    }

    /// Image used to boot the node.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.params.image = image.into();
        self
    }

    /// RAM configured on the node, in megabytes.
    pub fn ram(mut self, ram: u32) -> Self {
        self.params.ram = ram;
        self
    }

    /// QEMU version used to boot the node.
    pub fn qemu_version(mut self, qemu_version: impl Into<String>) -> Self {
        self.params.qemu_version = qemu_version.into();
        self
    }

    /// QEMU target architecture used to boot the node.
    pub fn qemu_arch(mut self, qemu_arch: impl Into<String>) -> Self {
        self.params.qemu_arch = qemu_arch.into();
        self
    }

    /// QEMU NIC model used for the interfaces of the node.
    pub fn qemu_nic(mut self, qemu_nic: impl Into<String>) -> Self {
        self.params.qemu_nic = qemu_nic.into();
        self
    }

    /// Custom options passed to QEMU to boot the node.
    pub fn qemu_options(mut self, qemu_options: impl Into<String>) -> Self {
        self.params.qemu_options = qemu_options.into();
        self
    }

    /// UUID configured for the node.
    pub fn uuid(mut self, uuid: impl Into<String>) -> Self {
        self.params.uuid = uuid.into();
        self
    }
}

impl EditNodeRequest<Vpcs> {
    /// Creates a request to edit a node of type [`NodeType::Vpcs`].
    pub fn vpcs(node: &Node) -> Result<Self> {
        Self::from_node(node)
    }
}
