//! Clients and models for managing users on the EVE-NG instance.

use crate::utils::{WireMap, empty_string_is_none, validate_name};
use crate::{Client, Error, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type to describe a user.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Expiration date as a UNIX timestamp, or `-1` if the account never
    /// expires.
    pub expiration: i64,
    /// A value representing a user profile. It is assigned automatically
    /// and unique for each user.
    pub pod: i8,
    /// Role of the user.
    pub role: String,
    /// Username of the user.
    pub username: String,
    /// Email address of the user.
    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub email: Option<String>,
    /// Path of the folder the user last viewed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    /// IP address the user last logged in from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// Path of the lab currently open for the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab: Option<String>,
    /// Full name of the user.
    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,
    /// Pod expiration date as a UNIX timestamp, or `-1` if the pod never
    /// expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pexpiration: Option<i64>,
    /// UNIX timestamp of the user's last login.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<u64>,
}

/// A client to manage users.
pub struct UsersClient {
    client: Client,
}

impl UsersClient {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Lists all users.
    pub async fn list(&self) -> Result<HashMap<String, User>> {
        Ok(self
            .client
            .get::<WireMap<String, User>>("users/")
            .await?
            .into_data()?
            .0)
    }

    /// Adds a new user.
    pub async fn add(&self, params: AddUserRequest) -> Result<UserClient> {
        self.client
            .post::<(), AddUserRequest>("users", &params)
            .await?;
        self.user(&params.username)
    }

    fn user(&self, username: impl Into<String>) -> Result<UserClient> {
        UserClient::new(self.client.clone(), username)
    }
}

/// Newtype for a valid username.
pub(crate) struct UserName(String);

impl std::fmt::Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl UserName {
    pub(crate) fn new(username: impl Into<String>) -> Result<Self> {
        let username = username.into();

        if username.is_empty() {
            return Err(Error::User("Username cannot be empty".to_string()));
        }

        if !validate_name(&username, &['-', '_']) {
            return Err(Error::User(format!(
                "Invalid username {}, must contain letters, digits, '-' and '_'.",
                &username
            )));
        }

        Ok(UserName(username))
    }
}

/// A client to manage a single user.
pub struct UserClient {
    client: Client,
    username: UserName,
}

impl UserClient {
    pub(crate) fn new(client: Client, username: impl Into<String>) -> Result<Self> {
        Ok(Self {
            client,
            username: UserName::new(username)?,
        })
    }

    /// Gets the user's details.
    pub async fn get(&self) -> Result<User> {
        self.client
            .get(&format!("users/{}", self.username))
            .await?
            .into_data()
    }

    /// Edits the user's details.
    pub async fn edit(&self, params: EditUserRequest) -> Result<()> {
        self.client
            .put::<(), EditUserRequest>(&format!("users/{}", self.username), &params)
            .await?;
        Ok(())
    }

    /// Deletes the user.
    pub async fn delete(self) -> Result<()> {
        self.client
            .delete::<()>(&format!("users/{}", self.username))
            .await?;
        Ok(())
    }
}

/// Request for adding a user.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AddUserRequest {
    username: String,
    password: String,
    role: String,
    expiration: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl AddUserRequest {
    /// Creates a new request for adding a user.
    ///
    /// `username` must only contain letters, digits, `-`, and `_` and
    /// `password` cannot be empty.
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Result<Self> {
        let username = UserName::new(username.into())?;
        let password = password.into();

        if password.is_empty() {
            return Err(Error::User("password cannot be empty".to_string()));
        }

        Ok(Self {
            username: username.to_string(),
            password,
            expiration: -1,
            role: "admin".to_string(),
            ..Default::default()
        })
    }

    /// Sets the email address of the user.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the account's expiration date, as a UNIX timestamp.
    /// Defaults to `-1`, meaning the user never expires.
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = expiration;
        self
    }

    /// Sets the full name of the user. Must only contain letters, digits,
    /// spaces, `-` and `_`.
    pub fn name(mut self, name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if !validate_name(&name, &['-', '_', ' ']) {
            return Err(Error::User(format!(
                "Invalid name '{}', must contain letters, digits, spaces, `-` and `_`.",
                &name,
            )));
        }
        self.name = Some(name);
        Ok(self)
    }

    /// Sets the role of the user. Defaults to `admin`.
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = role.into();
        self
    }
}

/// Request for editing a user.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditUserRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

impl EditUserRequest {
    /// Creates a new request for editing a user.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the password of the user.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the role of the user.
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Sets email address of the user.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the account's expiration date, as a UNIX timestamp.
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Sets the full name of the user.
    pub fn name(mut self, name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if !validate_name(&name, &['-', '_', ' ']) {
            return Err(Error::User(format!(
                "Invalid name '{}', must contain letters, digits, spaces, '-' and '_'.",
                &name,
            )));
        }
        self.name = Some(name);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_username() -> Result<()> {
        let result = UserName::new("test");
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn invalid_username() -> Result<()> {
        let result = UserName::new("test user");
        assert!(result.is_err());

        Ok(())
    }
}
