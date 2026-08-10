use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    #[serde(deserialize_with = "number_from_string")]
    pub expiration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab: Option<String>,
    pub name: String,
    #[serde(deserialize_with = "number_from_string")]
    pub pexpiration: i32,
    #[serde(deserialize_with = "number_from_string")]
    pub pod: i32,
    pub role: String,
    #[serde(deserialize_with = "number_from_string")]
    pub session: i32,
    pub username: String,
}
#[derive(Default, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub pod: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i32>,
}

#[derive(Default, Serialize)]
pub struct EditUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i32>,
}

pub struct Users {
    client: Client,
}

impl Users {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<HashMap<String, UserInfo>> {
        Ok(self
            .client
            .get::<WireMap<String, UserInfo>>("users/")
            .await?
            .into_data()?
            .0)
    }

    pub async fn add(&self, params: &CreateUserRequest) -> Result<User> {
        self.client
            .post::<(), CreateUserRequest>("users", params)
            .await?;
        Ok(User::new(self.client.clone(), &params.username))
    }
}

pub struct User {
    client: Client,
    username: String,
}

impl User {
    pub fn new(client: Client, username: &str) -> Self {
        Self {
            client,
            username: username.to_string(),
        }
    }

    pub async fn get(&self) -> Result<UserInfo> {
        self.client
            .get(&format!("users/{}", self.username))
            .await?
            .into_data()
    }

    pub async fn edit(&self, params: &EditUserRequest) -> Result<()> {
        self.client
            .put::<(), EditUserRequest>(&format!("users/{}", self.username), params)
            .await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        self.client
            .delete::<()>(&format!("users/{}", self.username))
            .await?;
        Ok(())
    }
}
