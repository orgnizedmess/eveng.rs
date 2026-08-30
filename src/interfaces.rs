///! Types and clients to manage interfaces of a node.
use crate::labs::{LabClient, LabPath};
use crate::networks::{AddNetworkRequest, EditNetworkRequest};
use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::{NodeClient, NodeType};
use crate::utils::map_or_seq;
use crate::{Client, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::marker::PhantomData;

#[derive(Debug, Serialize, Deserialize)]
pub struct EthernetInterface {
    pub name: String,
    pub network_id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerialInterface {
    pub name: String,
    pub remote_id: u32,
    pub remote_if: i32,
    pub remote_if_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interfaces {
    #[serde(deserialize_with = "map_or_seq")]
    pub ethernet: HashMap<u32, EthernetInterface>,
    #[serde(rename = "id")]
    pub node_id: u32,
    #[serde(deserialize_with = "map_or_seq")]
    pub serial: HashMap<u32, SerialInterface>,
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

pub enum InterfaceType {
    Ethernet,
    Serial,
}

mod private {
    pub trait Sealed {}
}

pub trait TypedInterface: private::Sealed {
    const INTERFACE_TYPE: InterfaceType;
}

impl private::Sealed for EthernetInterface {}
impl TypedInterface for EthernetInterface {
    const INTERFACE_TYPE: InterfaceType = InterfaceType::Ethernet;
}

impl private::Sealed for SerialInterface {}
impl TypedInterface for SerialInterface {
    const INTERFACE_TYPE: InterfaceType = InterfaceType::Serial;
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

    pub fn lab(&self) -> LabClient {
        LabClient::from_path(self.client.clone(), self.path.clone())
    }

    pub fn node(&self) -> NodeClient {
        NodeClient::new(self.client.clone(), self.path.clone(), self.id)
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

impl InterfaceClient<EthernetInterface> {
    pub(crate) fn ethernet(client: Client, path: LabPath, node_id: u32, id: u32) -> Self {
        Self::new(client, path, node_id, id)
    }

    /// Get an ethernet interface.
    pub async fn get(&self) -> Result<EthernetInterface> {
        InterfacesClient::new(self.client.clone(), self.path.clone(), self.node_id)
            .list()
            .await?
            .ethernet
            .remove(&self.id)
            .ok_or(Error::Interface(format!(
                "Ethernet interface '{}' not found",
                self.id
            )))
    }

    async fn is_connected(&self) -> Result<bool> {
        Ok(self.get().await?.network_id != 0)
    }

    /// Creates a point-to-point link between two nodes.
    pub async fn connect_to_node(&self, dest: &InterfaceClient<EthernetInterface>) -> Result<()> {
        if self.path.as_str() != dest.path.as_str() {
            return Err(Error::Interface(
                "Nodes from different labs cannot be connected.".to_string(),
            ));
        }

        if self.node_id == dest.node_id {
            return Err(Error::Interface(
                "Source and destination nodes cannot be the same.".to_string(),
            ));
        }

        if self.is_connected().await? {
            return Err(Error::Interface(
                "Source interface is already connected".to_string(),
            ));
        }

        if dest.is_connected().await? {
            return Err(Error::Interface(
                "Destination interface is already connected".to_string(),
            ));
        }

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

        // Setting the bridge to visibility 0 during creation causes errors,
        // hence it requires a separate request.
        bridge.edit(EditNetworkRequest::new().visibility(0)).await
    }

    /// Create a connection to the specified node's ethernet interface.
    pub async fn connect_to_network(&self, dest: &NetworkClient) -> Result<()> {
        if self.path.as_str() != dest.path.as_str() {
            return Err(Error::Interface(
                "Devices from different labs cannot be connected.".to_string(),
            ));
        }

        if self.is_connected().await? {
            return Err(Error::Interface(
                "Source interface is already connected".to_string(),
            ));
        }

        self.connect(dest.id.to_string()).await
    }

    /// Removes an existing connection on the ethernet interface.
    pub async fn disconnect(&self) -> Result<()> {
        self.lab().open().await?;

        let network_id = self.get().await?.network_id;
        let network = NetworkClient::new(self.client.clone(), self.path.clone(), network_id);
        let info = network.get().await?;

        if info.network_type == "bridge" && info.count == 2 && info.visibility == 0 {
            network.delete().await?;
        }

        self.connect(String::new()).await
    }
}

impl InterfaceClient<SerialInterface> {
    pub(crate) fn serial(client: Client, path: LabPath, node_id: u32, id: u32) -> Self {
        Self::new(client, path, node_id, id)
    }

    /// Gets a serial interface's details.
    pub async fn get(&self) -> Result<SerialInterface> {
        InterfacesClient::new(self.client.clone(), self.path.clone(), self.node_id)
            .list()
            .await?
            .serial
            .remove(&self.id)
            .ok_or(Error::Interface(format!(
                "Serial interface '{}' not found",
                self.id
            )))
    }

    async fn is_connected(&self) -> Result<bool> {
        Ok(self.get().await?.remote_id != 0)
    }

    /// Creates a point-to-point link between two nodes.
    pub async fn connect_to_node(&self, dest: &InterfaceClient<SerialInterface>) -> Result<()> {
        if self.path.as_str() != dest.path.as_str() {
            return Err(Error::Interface(
                "Nodes from different labs cannot be connected.".to_string(),
            ));
        }

        if self.node_id == dest.node_id {
            return Err(Error::Interface(
                "Source and destination nodes cannot be the same.".to_string(),
            ));
        }

        if self.is_connected().await? {
            return Err(Error::Interface(
                "Source interface is already connected".to_string(),
            ));
        }

        if dest.is_connected().await? {
            return Err(Error::Interface(
                "Destination interface is already connected".to_string(),
            ));
        }

        let remote_id = format!("{}:{}", dest.node_id, dest.id);
        self.connect(remote_id).await
    }

    /// Removes an existing connection on the serial interface.
    pub async fn disconnect(&self) -> Result<()> {
        self.connect(String::new()).await
    }
}
