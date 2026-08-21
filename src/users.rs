//! Clients and models for managing users on the EVE-NG instance.

use crate::utils::{WireMap, empty_string_is_none, validate_name};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// Expiration date as a UNIX timestamp or `-1` for no expiry.
    pub expiration: i64,

    /// A value representing a user profile. It is assigned automatically
    /// and unique for each user.
    pub pod: i8,

    pub role: String,

    /// Letters, digits and `-`/`_` only.
    pub username: String,

    /// The user's email address.
    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub email: Option<String>,

    /// Current folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,

    /// Last session IP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    /// Current lab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab: Option<String>,

    /// The user's full name.
    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub name: Option<String>,

    /// Pod expiration date as a UNIX timestamp or `-1` for no expiry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pexpiration: Option<i64>,

    /// Last session time as a UNIX timestamp.
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
        Ok(UserClient::new(self.client.clone(), &params.username))
    }
}

/// A client to manage a single user.
pub struct UserClient {
    client: Client,
    username: String,
}

impl UserClient {
    pub(crate) fn new(client: Client, username: &str) -> Self {
        Self {
            client,
            username: username.to_string(),
        }
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

/// Validates the specified username.
fn validate_username(username: impl Into<String>) -> Result<()> {
    validate_name(username, &['_', '-'])
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
    /// `username` must contain only letters, digits, `-`, and `_`.
    pub fn new(
        username: impl Into<String>,
        password: impl Into<String>,
        role: impl Into<String>,
    ) -> Result<Self> {
        let username = username.into();
        validate_username(&username)?;

        Ok(Self {
            username,
            password: password.into(),
            role: role.into(),
            expiration: -1,
            ..Default::default()
        })
    }

    /// Sets the user's email.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the date on which the user's validity expires, as a UNIX timestamp.
    /// Defaults to `-1`, meaning the user never expires.
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = expiration;
        self
    }

    /// Sets the user's display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
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

    /// Sets the user's password.
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the user's role.
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Sets the user's email.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets the date on which the user's validity expires, as a UNIX timestamp.
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Sets the user's display name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Clears the user's current full name.
    pub fn clear_name(mut self) -> Self {
        self.name = Some(String::new());
        self
    }

    /// Clears the user's current email address.
    pub fn clear_email(mut self) -> Self {
        self.email = Some(String::new());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn validate_user_name() {
        let result = validate_username("test");
        assert!(result.is_ok());

        let result = validate_username("test user");
        assert!(matches!(result, Err(Error::InvalidName(' '))));
    }
}
