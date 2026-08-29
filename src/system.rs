//! Client and models for system-level information about the EVE-NG instance.

use crate::utils::WireMap;
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    /// cached memory (percentage)
    pub cached: u8,
    /// CPU usage (percentage)
    pub cpu: u8,
    /// Cpulimit status (enabled/disabled/unsupported)
    pub cpulimit: String,
    /// Disk usage (percentage)
    pub disk: u8,
    // Running Docker wrappers
    pub docker: u32,
    /// Running Dynamips wrappers
    pub dynamips: u32,
    /// Running IOL wrappers
    pub iol: u32,
    /// KSM status (enabled/disabled/unsupported)
    pub ksm: String,
    /// Memory usage (percentage)
    pub mem: u8,
    /// Running QEMU wrappers
    pub qemu: u32,
    pub qemu_version: String,
    /// Swap usage (percentage)
    pub swap: u8,
    /// UKSM status (enabled/disabled/unsupported)
    pub uksm: String,
    /// EVE-NG version
    pub version: String,
    /// Running VPCS wrappers
    pub vpcs: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub email: String,
    /// Current folder
    pub folder: String,
    pub html5: i8,
    pub lang: String,
    pub name: String,
    pub role: String,
    pub tenant: u32,
    pub username: String,
    /// Current lab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab: Option<String>,
}

/// A client for system-level information.
pub struct SystemClient {
    client: Client,
}

impl SystemClient {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Gets the current system status.
    pub async fn status(&self) -> Result<SystemStatus> {
        self.client.get("status").await?.into_data()
    }

    /// Gets the currently authenticated user's details.
    pub async fn auth_status(&self) -> Result<AuthStatus> {
        self.client.get("auth").await?.into_data()
    }

    /// Lists available network types.
    pub async fn network_types(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/networks")
            .await?
            .into_data()?
            .0)
    }

    /// Lists available user roles.
    pub async fn user_roles(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/roles")
            .await?
            .into_data()?
            .0)
    }
}
