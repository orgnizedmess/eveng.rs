//! Clients and models for managing networks within a lab.

use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
    /// Number of connected nodes.
    pub count: u32,

    pub icon: String,
    pub left: u32,
    pub name: String,
    pub top: u32,

    #[serde(rename = "type")]
    pub network_type: String,

    #[serde(deserialize_with = "number_from_string")]
    pub visibility: u8,

    // appears in /networks, not in /networks/{id}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
}

pub struct NetworksClient {
    client: Client,
    path: String,
}

impl NetworksClient {
    pub(crate) fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
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
        Ok(NetworkClient::new(self.client.clone(), &self.path, resp.id))
    }
}

pub struct NetworkClient {
    pub(crate) client: Client,
    pub(crate) path: String,
    pub(crate) id: u32,
}

impl NetworkClient {
    pub(crate) fn new(client: Client, path: impl Into<String>, id: u32) -> Self {
        Self {
            client,
            path: path.into(),
            id,
        }
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AddNetworkRequest {
    #[serde(rename = "type")]
    network_type: String,
    visibility: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    // API shows percentage values but regular ints works on my instance
    #[serde(skip_serializing_if = "Option::is_none")]
    left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    // API shows percentage values but regular ints works on my instance
    #[serde(skip_serializing_if = "Option::is_none")]
    top: Option<u32>,
}

impl AddNetworkRequest {
    pub fn new(network_type: impl Into<String>) -> Self {
        Self {
            network_type: network_type.into(),
            visibility: 1,
            ..Default::default()
        }
    }

    pub fn visibility(mut self, visibility: u8) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = Some(left);
        self.top = Some(top);
        self
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditNetworkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub network_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<u8>,
}

impl EditNetworkRequest {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn position(mut self, left: u32, top: u32) -> Self {
        self.left = Some(left);
        self.top = Some(top);
        self
    }

    pub fn visibility(mut self, visibility: u8) -> Self {
        self.visibility = Some(visibility);
        self
    }
}
