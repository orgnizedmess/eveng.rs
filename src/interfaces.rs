//! Types and clients to manage interfaces of a node.

use crate::labs::{LabClient, LabPath};
use crate::networks::{AddNetworkRequest, EditNetworkRequest};
use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::{NodeClient, NodeType};
use crate::utils::{map_or_seq, private::Sealed};
use crate::{Client, Error, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

/// Type to describe an Ethernet interface.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ethernet {
    /// Display name of the interface.
    pub name: String,
    /// ID of the network the interface is connected to. Set to `0` if not
    /// connected.
    pub network_id: u32,
}

/// Type to describe a Serial interface.
#[derive(Debug, Serialize, Deserialize)]
pub struct Serial {
    /// Display name of the interface.
    pub name: String,
    /// ID of the node the interface is connected to. Set to `0` if not
    /// connected.
    pub remote_id: u32,
    /// ID of the connected node's interface. Set to `0` if not
    /// connected.
    pub remote_if: i32,
    /// Name of the connected node's interface. Empty if not connected.
    pub remote_if_name: String,
}

/// Type to describe the list of interfaces of a node.
#[derive(Debug, Serialize, Deserialize)]
pub struct Interfaces {
    /// List of ethernet interfaces.
    #[serde(deserialize_with = "map_or_seq")]
    pub ethernet: HashMap<u32, Ethernet>,
    /// ID of the node.
    #[serde(rename = "id")]
    pub node_id: u32,
    /// List of serial interfaces.
    #[serde(deserialize_with = "map_or_seq")]
    pub serial: HashMap<u32, Serial>,
    /// Type of the node.
    #[serde(rename = "sort")]
    pub node_type: NodeType,
}

/// A client to manage interfaces.
pub struct InterfacesClient {
    client: Client,
    path: LabPath,
    node_id: u32,
}

impl InterfacesClient {
    pub(crate) fn new(client: Client, path: LabPath, node_id: u32) -> Self {
        Self {
            client,
            path,
            node_id,
        }
    }

    /// Gets a node's interfaces.
    pub async fn list(&self) -> Result<Interfaces> {
        self.client
            .get(&format!(
                "labs{}/nodes/{}/interfaces",
                self.path, self.node_id
            ))
            .await?
            .into_data()
    }
}

/// Type of the interface.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceType {
    Ethernet,
    Serial,
}

impl std::fmt::Display for InterfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            InterfaceType::Ethernet => "ethernet",
            InterfaceType::Serial => "serial",
        };

        f.write_str(value)
    }
}

/// Trait that binds static interface types to [`InterfaceType`].
pub trait TypedInterface: Sealed + Sized {
    /// Type of the interface.
    const INTERFACE_TYPE: InterfaceType;

    /// Returns an interface from a list of interfaces.
    fn take(ifaces: Interfaces, id: u32) -> Option<Self>;
    /// Checks if the interface is connected to another node or network.
    fn is_connected(&self) -> bool;
}

impl Sealed for Ethernet {}
impl TypedInterface for Ethernet {
    const INTERFACE_TYPE: InterfaceType = InterfaceType::Ethernet;

    fn take(mut ifaces: Interfaces, id: u32) -> Option<Self> {
        ifaces.ethernet.remove(&id)
    }

    fn is_connected(&self) -> bool {
        self.network_id != 0
    }
}

impl Sealed for Serial {}
impl TypedInterface for Serial {
    const INTERFACE_TYPE: InterfaceType = InterfaceType::Serial;

    fn take(mut ifaces: Interfaces, id: u32) -> Option<Self> {
        ifaces.serial.remove(&id)
    }

    fn is_connected(&self) -> bool {
        self.remote_id != 0
    }
}

/// A client to manage a single interface.
pub struct InterfaceClient<T> {
    client: Client,
    path: LabPath,
    node_id: u32,
    id: u32,
    _marker: PhantomData<T>,
}

impl<T: TypedInterface> InterfaceClient<T> {
    pub(crate) fn new(client: Client, path: LabPath, node_id: u32, id: u32) -> Self {
        Self {
            client,
            path,
            node_id,
            id,
            _marker: PhantomData,
        }
    }

    fn interfaces(&self) -> InterfacesClient {
        InterfacesClient::new(self.client.clone(), self.path.clone(), self.node_id)
    }

    fn lab(&self) -> LabClient {
        LabClient::from_path(self.client.clone(), self.path.clone())
    }

    /// Gets the interface's details.
    pub async fn get(&self) -> Result<T> {
        let ifaces = self.interfaces().list().await?;

        T::take(ifaces, self.id).ok_or(Error::Client(
            "Cannot find interface for the selected node.".to_string(),
        ))
    }

    /// Checks if the interface is connected to another node or network.
    pub async fn is_connected(&self) -> Result<bool> {
        Ok(self.get().await?.is_connected())
    }

    async fn ensure_connectable(&self, dest: &Self) -> Result<()> {
        if self.client.base_url != dest.client.base_url {
            return Err(Error::Client(
                "Nodes from different clients cannot be connected.".to_string(),
            ));
        }

        if self.path.as_str() != dest.path.as_str() {
            return Err(Error::Client(
                "Nodes from different labs cannot be connected.".to_string(),
            ));
        }

        if self.node_id == dest.node_id {
            return Err(Error::Client(
                "Source and destination nodes cannot be the same.".to_string(),
            ));
        }

        if self.is_connected().await? {
            return Err(Error::Client(
                "Source interface is already connected".to_string(),
            ));
        }

        if dest.is_connected().await? {
            return Err(Error::Client(
                "Destination interface is already connected".to_string(),
            ));
        }

        Ok(())
    }

    async fn connect(&self, dest_id: String) -> Result<()> {
        self.lab().open().await?;

        let req = serde_json::json!({self.id.to_string(): dest_id});
        self.client
            .put::<(), serde_json::Value>(
                &format!("labs{}/nodes/{}/interfaces", self.path, self.node_id),
                &req,
            )
            .await?;
        Ok(())
    }
}

impl InterfaceClient<Ethernet> {
    pub(crate) fn ethernet(client: Client, path: LabPath, node_id: u32, id: u32) -> Self {
        Self::new(client, path, node_id, id)
    }

    /// Creates a point-to-point connection between ethernet interfaces of two
    /// nodes.
    pub async fn connect_to_node(&self, dest: &InterfaceClient<Ethernet>) -> Result<()> {
        self.ensure_connectable(dest).await?;

        let src = NodeClient::new(self.client.clone(), self.path.clone(), self.node_id)
            .get()
            .await?;

        let bridge = NetworksClient::new(self.client.clone(), self.path.clone())
            .add(AddNetworkRequest::new("bridge").name(format!("Net-{}iface{}", src.name, self.id)))
            .await?;

        self.connect(bridge.id.to_string()).await?;

        if let Err(e) = dest.connect(bridge.id.to_string()).await {
            bridge.delete().await?;
            return Err(e);
        }

        // Making the bridge invisible during creation causes errors,
        // hence it requires a separate request
        bridge.edit(EditNetworkRequest::new().visibility(0)).await
    }

    /// Creates a connection between a node and a network.
    pub async fn connect_to_network(&self, dest: &NetworkClient) -> Result<()> {
        if self.client.base_url != dest.client.base_url {
            return Err(Error::Client(
                "Nodes from different clients cannot be connected.".to_string(),
            ));
        }

        if self.path.as_str() != dest.path.as_str() {
            return Err(Error::Client(
                "Devices from different labs cannot be connected.".to_string(),
            ));
        }

        if self.is_connected().await? {
            return Err(Error::Client(
                "Source interface is already connected".to_string(),
            ));
        }

        self.connect(dest.id.to_string()).await
    }

    /// Removes an existing connection on the ethernet interface.
    pub async fn disconnect(&self) -> Result<()> {
        let iface = self.get().await?;
        if !iface.is_connected() {
            return Ok(());
        }

        let network = NetworkClient::new(self.client.clone(), self.path.clone(), iface.network_id);
        let info = network.get().await?;

        if info.network_type == "bridge" && info.count == 2 && info.visibility == 0 {
            // Deleting the network is sufficient for a node -> node connection
            network.delete().await
        } else {
            self.connect(String::new()).await
        }
    }
}

impl InterfaceClient<Serial> {
    pub(crate) fn serial(client: Client, path: LabPath, node_id: u32, id: u32) -> Self {
        Self::new(client, path, node_id, id)
    }

    /// Creates a point-to-point connection between serial interfaces
    /// of two nodes.
    pub async fn connect_to_node(&self, dest: &InterfaceClient<Serial>) -> Result<()> {
        self.ensure_connectable(dest).await?;

        let remote_id = format!("{}:{}", dest.node_id, dest.id);
        self.connect(remote_id).await
    }

    /// Removes an existing connection on the serial interface.
    pub async fn disconnect(&self) -> Result<()> {
        self.connect(String::new()).await
    }
}
