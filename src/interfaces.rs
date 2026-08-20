use crate::networks::{CreateNetworkRequest, EditNetworkRequest};
use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::NodeClient;
use crate::utils::map_or_seq;
use crate::{Client, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Interface {
    Ethernet(EthernetInterface),
    Serial(SerialInterface),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EthernetInterface {
    pub name: String,
    pub network_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SerialInterface {
    pub name: String,
    pub remote_id: i32,
    pub remote_if: i32,
    pub remote_if_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interfaces {
    #[serde(deserialize_with = "map_or_seq")]
    pub ethernet: HashMap<i32, EthernetInterface>,
    #[serde(rename = "id")]
    pub node_id: i32,
    #[serde(deserialize_with = "map_or_seq")]
    pub serial: HashMap<i32, SerialInterface>,
    #[serde(rename = "sort")]
    pub node_type: String,
}

pub struct InterfacesClient {
    client: Client,
    path: String,
    node_id: i32,
}

impl InterfacesClient {
    pub(crate) fn new(client: Client, path: impl Into<String>, node_id: i32) -> Self {
        Self {
            client,
            path: path.into(),
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

pub struct InterfaceClient {
    client: Client,
    path: String,
    node_id: i32,
    id: i32,
    iface_type: InterfaceType,
}

impl InterfaceClient {
    pub(crate) fn new(
        client: Client,
        path: impl Into<String>,
        node_id: i32,
        id: i32,
        iface_type: InterfaceType,
    ) -> Self {
        Self {
            client,
            path: path.into(),
            node_id,
            id,
            iface_type,
        }
    }

    pub async fn connect_to_node(&self, dest: &InterfaceClient) -> Result<()> {
        match (&self.iface_type, &dest.iface_type) {
            (InterfaceType::Ethernet, InterfaceType::Ethernet) => {
                let src_node = NodeClient::new(self.client.clone(), &self.path, self.node_id)
                    .get()
                    .await?;

                let bridge = NetworksClient::new(self.client.clone(), &self.path)
                    .add(&CreateNetworkRequest {
                        network_type: "bridge".to_string(),
                        visibility: 1,
                        icon: None,
                        left: None,
                        name: Some(format!("Net-{}iface{}", src_node.name, self.id)),
                        //postfix: None,
                        top: None,
                    })
                    .await?;

                self.connect_to_network(&bridge).await?;
                dest.connect_to_network(&bridge).await?;

                bridge
                    .edit(&EditNetworkRequest {
                        network_type: None,
                        visibility: Some(0),
                        icon: None,
                        left: None,
                        name: None,
                        // postfix: None,
                        top: None,
                    })
                    .await
            }
            (InterfaceType::Serial, InterfaceType::Serial) => {
                let serial_id = format!("{}:{}", dest.node_id, dest.id);
                self.connect(serial_id).await
            }
            _ => Err(Error::Invalid("Mismatched interface types".to_string())),
        }
    }

    pub async fn connect_to_network(&self, dest: &NetworkClient) -> Result<()> {
        if !matches!(self.iface_type, InterfaceType::Ethernet) {
            return Err(Error::MissingData);
        }
        self.connect(dest.id.to_string()).await
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

    pub async fn disconnect(&self) -> Result<()> {
        match self.iface_type {
            InterfaceType::Ethernet => {
                let network_id =
                    InterfacesClient::new(self.client.clone(), &self.path, self.node_id)
                        .list()
                        .await?
                        .ethernet
                        .remove(&self.id)
                        .ok_or(Error::Invalid("Interface not found".to_string()))?
                        .network_id;

                let network = NetworkClient::new(self.client.clone(), &self.path, network_id);
                if network.get().await?.network_type == "bridge" {
                    network.delete().await?;
                    Ok(())
                } else {
                    self.connect(String::new()).await
                }
            }
            InterfaceType::Serial => self.connect(String::new()).await,
        }
    }
}
