use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub count: i32,
    pub icon: String,
    // appears in /networks, not in /networks/{id}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub left: i32,
    pub name: String,
    pub top: i32,
    #[serde(rename = "type")]
    pub network_type: String,
    #[serde(deserialize_with = "number_from_string")]
    pub visibility: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNetworkRequest {
    pub count: i32,
    #[serde(rename = "type")]
    pub network_type: String,
    // not visible (haha) in the create page but visible (heheh) in the API request
    // Create will return successfully without it, but not actually list any nodes
    // WAT
    pub visibility: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    // API shows percentage values but regular ints works on my instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postfix: Option<i32>,
    // API shows percentage values but regular ints works on my instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNetworkResponse {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditNetworkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub network_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postfix: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteNetworkResponse {
    #[serde(deserialize_with = "number_from_string")]
    pub id: i32,
    pub count: i32,
    pub left: i32,
    pub name: String,
    pub top: i32,
    #[serde(rename = "type")]
    pub network_type: String,
}

pub struct Networks {
    client: Client,
    path: String,
}

impl Networks {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    pub async fn list(&self) -> Result<HashMap<i32, NetworkInfo>> {
        Ok(self
            .client
            .get::<WireMap<i32, NetworkInfo>>(&format!("labs{}/networks", self.path))
            .await?
            .into_data()?
            .0)
    }

    pub async fn add(&self, params: &CreateNetworkRequest) -> Result<Network> {
        let resp: CreateNetworkResponse = self
            .client
            .post(&format!("labs{}/networks", self.path), params)
            .await?
            .into_data()?;
        Ok(Network::new(self.client.clone(), &self.path, resp.id))
    }
}

pub struct Network {
    client: Client,
    path: String,
    id: i32,
}

impl Network {
    pub fn new(client: Client, path: &str, id: i32) -> Self {
        Self {
            client,
            path: path.to_string(),
            id,
        }
    }

    pub async fn get(&self) -> Result<NetworkInfo> {
        self.client
            .get(&format!("labs{}/networks/{}", self.path, self.id))
            .await?
            .into_data()
    }

    pub async fn edit(&self, params: &EditNetworkRequest) -> Result<()> {
        self.client
            .put::<(), EditNetworkRequest>(
                &format!("labs{}/networks/{}", self.path, self.id),
                params,
            )
            .await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<DeleteNetworkResponse> {
        self.client
            .delete(&format!("labs{}/networks/{}", self.path, self.id))
            .await?
            .into_data()
    }
}
