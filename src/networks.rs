//! Types and clients for managing networks within a lab.

use crate::labs::LabPath;
use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type to describe a network in a lab.
#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    /// Number of connected nodes.
    pub count: u32,
    /// Icon used to display the network in the lab.
    pub icon: String,
    /// Left margin of the node.
    pub left: u32,
    /// Name used to display the network in the lab.
    pub name: String,
    /// Top margin of the network.
    pub top: u32,
    /// Type of the network.
    #[serde(rename = "type")]
    pub network_type: String,
    /// Visbility of the network in the lab.
    #[serde(deserialize_with = "number_from_string")]
    pub visibility: u8,
    /// Identifier of the network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
}

/// A client to manage networks.
pub struct NetworksClient {
    client: Client,
    path: LabPath,
}

impl NetworksClient {
    pub(crate) fn new(client: Client, path: LabPath) -> Self {
        Self { client, path }
    }

    /// Lists all networks.
    pub async fn list(&self) -> Result<HashMap<i32, Network>> {
        Ok(self
            .client
            .get::<WireMap<i32, Network>>(&format!("labs{}/networks", self.path))
            .await?
            .into_data()?
            .0)
    }

    /// Adds a new network to the lab.
    pub async fn add(&self, params: AddNetworkRequest) -> Result<NetworkClient> {
        #[derive(Deserialize)]
        struct CreateNetworkResponse {
            id: u32,
        }

        let resp: CreateNetworkResponse = self
            .client
            .post(&format!("labs{}/networks", self.path), &params)
            .await?
            .into_data()?;

        Ok(self.network(resp.id))
    }

    fn network(&self, id: u32) -> NetworkClient {
        NetworkClient::new(self.client.clone(), self.path.clone(), id)
    }
}

/// A client to manage a single network.
pub struct NetworkClient {
    pub(crate) client: Client,
    pub(crate) path: LabPath,
    pub(crate) id: u32,
}

impl NetworkClient {
    pub(crate) fn new(client: Client, path: LabPath, id: u32) -> Self {
        Self { client, path, id }
    }

    /// Gets the network's details.
    pub async fn get(&self) -> Result<Network> {
        self.client
            .get(&format!("labs{}/networks/{}", self.path, self.id))
            .await?
            .into_data()
    }

    /// Updates the network's details.
    pub async fn edit(&self, params: EditNetworkRequest) -> Result<()> {
        self.client
            .put::<(), EditNetworkRequest>(
                &format!("labs{}/networks/{}", self.path, self.id),
                &params,
            )
            .await?;
        Ok(())
    }

    /// Deletes the network.
    pub async fn delete(self) -> Result<()> {
        #[derive(Debug, Serialize, Deserialize)]
        struct DeleteNetworkResponse {
            #[serde(deserialize_with = "number_from_string")]
            id: u32,
            count: i32,
            left: i32,
            name: String,
            top: i32,
            #[serde(rename = "type")]
            network_type: String,
        }

        let _: DeleteNetworkResponse = self
            .client
            .delete(&format!("labs{}/networks/{}", self.path, self.id))
            .await?
            .into_data()?;
        Ok(())
    }
}

/// Request to add a network.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AddNetworkRequest {
    left: u32,
    #[serde(rename = "type")]
    network_type: String,
    top: u32,
    visibility: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl AddNetworkRequest {
    /// Creates a request to add a network.
    pub fn new(network_type: impl Into<String>) -> Self {
        Self {
            left: 0,
            network_type: network_type.into(),
            top: 0,
            visibility: 1,
            ..Self::default()
        }
    }

    /// Icon used to display the network in the lab.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Name used to display the network in the lab.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Position of the node in the lab.
    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = left;
        self.top = top;
        self
    }

    /// Visiblity of the network in the lab.
    pub fn visibility(mut self, visibility: u8) -> Self {
        self.visibility = visibility;
        self
    }
}

/// Request to edit a network.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditNetworkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    network_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility: Option<u8>,
}

impl EditNetworkRequest {
    /// Creates a request to edit a network.
    pub fn new() -> Self {
        Self::default()
    }

    /// Icon used to display the network in the lab.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Name used to display the network in the lab.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Type of the network.
    pub fn network_type(mut self, network_type: impl Into<String>) -> Self {
        self.network_type = Some(network_type.into());
        self
    }

    /// Position of the node in the lab.
    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = Some(left);
        self.top = Some(top);
        self
    }

    /// Visiblity of the network in the lab.
    pub fn visibility(mut self, visibility: u8) -> Self {
        self.visibility = Some(visibility);
        self
    }
}
