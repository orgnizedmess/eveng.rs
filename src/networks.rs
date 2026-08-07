use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Network {
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

pub type Networks = HashMap<i32, Network>;

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

pub type NetworkTypes = HashMap<String, String>;

impl Client {
    pub async fn networks(&self) -> Result<Networks> {
        Ok(self
            .get::<WireMap<i32, Network>>(&format!("/labs/{}/networks", self.lab_path))
            .await?
            .into_data()?
            .0)
    }

    pub async fn network(&self, id: i32) -> Result<Network> {
        self.get(&format!("/labs/{}/networks/{}", self.lab_path, id))
            .await?
            .into_data()
    }

    pub async fn add_network(&self, params: CreateNetworkRequest) -> Result<CreateNetworkResponse> {
        self.post::<CreateNetworkResponse, CreateNetworkRequest>(
            &format!("/labs/{}/networks", self.lab_path),
            params,
        )
        .await?
        .into_data()
    }

    pub async fn edit_network(&self, id: i32, params: EditNetworkRequest) -> Result<()> {
        self.put::<(), EditNetworkRequest>(
            &format!("/labs/{}/networks/{}", self.lab_path, id),
            params,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_network(&self, id: i32) -> Result<DeleteNetworkResponse> {
        self.delete(&format!("/labs/{}/networks/{}", self.lab_path, id))
            .await?
            .into_data()
    }

    pub async fn network_types(&self) -> Result<NetworkTypes> {
        Ok(self
            .get::<WireMap<String, String>>("/list/networks")
            .await?
            .into_data()?
            .0)
    }
}
