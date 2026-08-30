///! Types and clients to manage interfaces of a node.
use crate::labs::LabPath;
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

    async fn connect(&self, dest_id: String) -> Result<()> {
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

    /// Creates a point-to-point link between two nodes.
    pub async fn connect_to_node(
        &self,
        dest: &InterfaceClient<EthernetInterface>,
    ) -> Result<()> {
        let src_node = NodeClient::new(self.client.clone(), self.path.clone(), self.node_id)
            .get()
            .await?;

        let bridge = NetworksClient::new(self.client.clone(), self.path.clone())
            .add(
                AddNetworkRequest::new("bridge")
                    .name(format!("Net-{}iface{}", src_node.name, self.id)),
            )
            .await?;

        self.connect(bridge.id.to_string()).await?;
        dest.connect(bridge.id.to_string()).await?;

        bridge.edit(EditNetworkRequest::new().visibility(0)).await
    }

    /// Create a connection to the specified node's ethernet interface.
    ///
    /// This will overwrite an existing connection if one exists.
    pub async fn connect_to_network(&self, dest: &NetworkClient) -> Result<()> {
        self.connect(dest.id.to_string()).await
    }

    /// Removes an existing connection on the ethernet interface.
    pub async fn disconnect(&self) -> Result<()> {
        let network_id = self.get().await?.network_id;

        let network = NetworkClient::new(self.client.clone(), self.path.clone(), network_id);

        if network.get().await?.network_type == "bridge" {
            network.delete().await?;
            Ok(())
        } else {
            self.connect(String::new()).await
        }
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

    /// Creates a point-to-point link between two nodes.
    pub async fn connect_to_node(
        &self,
        dest: &InterfaceClient<SerialInterface>,
    ) -> Result<()> {
        let remote_id = format!("{}:{}", dest.node_id, dest.id);
        self.connect(remote_id).await
    }

    /// Removes an existing connection on the serial interface.
    pub async fn disconnect(&self) -> Result<()> {
        self.connect(String::new()).await
    }
}
