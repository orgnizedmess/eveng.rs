use crate::{Client, Result};
use crate::utils::number_from_string;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct User {
    email: String,
    #[serde(deserialize_with = "number_from_string")]
    expiration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<String>,
    ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    lab: Option<String>,
    name: String,
    #[serde(deserialize_with = "number_from_string")]
    pexpiration: i32,
    #[serde(deserialize_with = "number_from_string")]
    pod: i32,
    role: String,
    #[serde(deserialize_with = "number_from_string")]
    session: i32,
    username: String,
}
#[derive(Default, Serialize)]
pub struct CreateUserRequest {
    username: String,
    password: String,
    // Validate by matching with /api/list/roles?
    role: String,
    pod: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<i32>,
}

#[derive(Default, Serialize)]
pub struct EditUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pod: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<i32>,
}

impl Client {
    pub async fn users(&self) -> Result<HashMap<String, serde_json::Value>> {
        self.get("users/").await?.into_data()
    }

    pub async fn user(&self, username: &str) -> Result<User> {
        self.get(&format!("users/{}", username)).await?.into_data()
    }

    pub async fn create_user(&self, params: &CreateUserRequest) -> Result<()> {
        self.post::<(), CreateUserRequest>("users", params).await?;
        Ok(())
    }

    pub async fn edit_user(&self, username: &str, params: &EditUserRequest) -> Result<()> {
        self.put::<(), EditUserRequest>(&format!("users/{}", username), params)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, username: &str) -> Result<()> {
        self.delete::<()>(&format!("users/{}", username)).await?;
        Ok(())
    }

    pub async fn user_roles(&self) -> Result<serde_json::Value> {
        self.get("list/roles").await?.into_data()
    }
}
