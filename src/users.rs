use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct User {
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
    // Validate by matching with /api/list/roles?
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

pub type Users = HashMap<String, User>;
pub type UserRoles = HashMap<String, String>;

impl Client {
    pub async fn users(&self) -> Result<Users> {
        Ok(self
            .get::<WireMap<String, User>>("users/")
            .await?
            .into_data()?
            .0)
    }

    pub async fn user(&self, username: &str) -> Result<User> {
        self.get(&format!("users/{}", username)).await?.into_data()
    }

    pub async fn create_user(&self, params: CreateUserRequest) -> Result<()> {
        self.post::<(), CreateUserRequest>("users", params).await?;
        Ok(())
    }

    pub async fn edit_user(&self, username: &str, params: EditUserRequest) -> Result<()> {
        self.put::<(), EditUserRequest>(&format!("users/{}", username), params)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, username: &str) -> Result<()> {
        self.delete::<()>(&format!("users/{}", username)).await?;
        Ok(())
    }

    pub async fn user_roles(&self) -> Result<UserRoles> {
        Ok(self
            .get::<WireMap<String, String>>("list/roles")
            .await?
            .into_data()?
            .0)
    }
}
