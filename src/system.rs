//! Types and client for system-level information about the EVE-NG instance.

use crate::templates::{TemplateClient, TemplatesClient};
use crate::utils::WireMap;
use crate::{Client, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type to describe the status of the system running the EVE-NG instance.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Percentage of total RAM that is cached.
    pub cached: u8,
    /// Percentage of total CPU currently in use.
    pub cpu: u8,
    /// Cpulimit status (enabled/disabled/unsupported).
    pub cpulimit: String,
    /// Percentage of total disk space currently in use.
    pub disk: u8,
    /// Number of running Docker nodes.
    pub docker: u32,
    /// Number of running Dynamips nodes.
    pub dynamips: u32,
    /// Number unning IOL nodes.
    pub iol: u32,
    /// KSM status (enabled/disabled/unsupported).
    pub ksm: String,
    /// Percentage of total RAM currently in use.
    pub mem: u8,
    /// Number of running QEMU nodes.
    pub qemu: u32,
    /// Installed QEMU version on the host.
    pub qemu_version: String,
    /// Percentage of the total swap currently in use.
    pub swap: u8,
    /// UKSM status (enabled/disabled/unsupported).
    pub uksm: String,
    /// EVE-NG version.
    pub version: String,
    /// Number of running VPCS nodes.
    pub vpcs: u32,
}

/// Type to describe the status of the currently authenticated user.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    /// Email address of the user.
    pub email: String,
    /// Path of the folder the user last viewed.
    pub folder: String,
    /// A flag to indicate if the html5 console is in use.
    pub html5: i8,
    /// Language select
    pub lang: String,
    /// Full name of the user.
    pub name: String,
    /// Role of the user.
    pub role: String,
    /// Tenant ID of the user.
    pub tenant: u32,
    /// Username of the user.
    pub username: String,
    /// Path of the lab currently open for the user.
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

    /// Returns a client to manage node templates.
    pub fn node_templates(&self) -> TemplatesClient {
        TemplatesClient::new(self.client.clone())
    }

    /// Returns a client to manage a single node template.
    pub fn node_template(&self, name: impl Into<String>) -> TemplateClient {
        TemplateClient::new(self.client.clone(), name)
    }
}
