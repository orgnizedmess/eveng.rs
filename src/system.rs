//! System-wide endpoints

use crate::utils::{WireMap, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Current system status
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    /// cached memory (percentage)
    pub cached: u8,
    /// CPU usage (percentage)
    pub cpu: u8,
    /// Disk usage (percentage)
    pub disk: u8,
    /// Running Dynamips wrappers
    pub dynamips: u32,
    /// Running IOL wrappers
    pub iol: u32,
    /// Memory usage (percentage)
    pub mem: u8,
    /// Running QEMU wrappers
    pub qemu: u32,
    pub qemu_version: String,
    /// Swap usage (percentage)
    pub swap: u8,
    /// EVE-NG version
    pub version: String,
}

/// Information about the currently authenticated user
#[derive(Serialize, Deserialize)]
pub struct AuthInfo {
    pub email: String,
    /// Path of the currently open lab
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lab: Option<String>,
    pub folder: String,
    pub html5: u8,
    pub lang: String,
    pub name: String,
    pub role: String,
    #[serde(deserialize_with = "number_from_string")]
    pub tenant: u32,
    pub username: String,
}

/// A list of available node templates
pub type NodeTemplates = HashMap<String, String>;

/// List of options in a node template and their default values
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTemplate {
    pub description: String,
    pub options: HashMap<String, Value>,
    #[serde(rename = "type")]
    pub node_type: String,
}

/// A list of user roles
pub type UserRoles = HashMap<String, String>;

/// A list of available network types
pub type NetworkTypes = HashMap<String, String>;

/// System-wide endpoints
pub struct System {
    client: Client,
}

impl System {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get the current system status
    pub async fn status(&self) -> Result<SystemStatus> {
        self.client.get("status").await?.into_data()
    }

    /// Get information about the currently authenticated user
    pub async fn auth_info(&self) -> Result<AuthInfo> {
        self.client.get("auth").await?.into_data()
    }

    /// List node templates
    pub async fn node_templates(&self) -> Result<HashMap<String, String>> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/templates/")
            .await?
            .into_data()?
            .0)
    }

    /// Get information about a specific node template
    pub async fn node_template(&self, template: &str) -> Result<NodeTemplate> {
        self.client
            .get(&format!("list/templates/{}", template))
            .await?
            .into_data()
    }

    /// List network types
    pub async fn network_types(&self) -> Result<NetworkTypes> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/networks")
            .await?
            .into_data()?
            .0)
    }

    /// List user roles
    pub async fn user_roles(&self) -> Result<UserRoles> {
        Ok(self
            .client
            .get::<WireMap<String, String>>("list/roles")
            .await?
            .into_data()?
            .0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_system_status() {
        let json = r#"
        {
          "cached": 64,
          "cpu": 4,
          "disk": 4,
          "dynamips": 0,
          "iol": 0,
          "mem": 17,
          "qemu": 4,
          "qemu_version": "2.4.0",
          "swap": 0,
          "version": "6.2.0-4"
        }
        "#;

        let status: SystemStatus = serde_json::from_str(json).unwrap();

        assert_eq!(status.cached, 64);
        assert_eq!(status.cpu, 4);
        assert_eq!(status.disk, 4);
        assert_eq!(status.dynamips, 0);
        assert_eq!(status.iol, 0);
        assert_eq!(status.mem, 17);
        assert_eq!(status.qemu, 4);
        assert_eq!(status.qemu_version, "2.4.0");
        assert_eq!(status.swap, 0);
        assert_eq!(status.version, "6.2.0-4");
    }

    #[test]
    fn deserialize_auth_info() {
        let json = r#"
        {
          "email": "root@localhost",
          "lab": null,
          "folder": "/",
          "html5": 1,
          "lang": "en",
          "name": "Eve-NG Administrator",
          "role": "admin",
          "tenant": 0,
          "username": "admin"
        }
        "#;

        let auth_info: AuthInfo = serde_json::from_str(json).unwrap();

        assert_eq!(auth_info.email, "root@localhost");
        assert_eq!(auth_info.lab, None);
        assert_eq!(auth_info.folder, "/");
        assert_eq!(auth_info.html5, 1);
        assert_eq!(auth_info.lang, "en");
        assert_eq!(auth_info.name, "Eve-NG Administrator");
        assert_eq!(auth_info.role, "admin");
        assert_eq!(auth_info.tenant, 0);
        assert_eq!(auth_info.username, "admin");
    }

    #[test]
    fn deserialize_node_templates() {
        let json = r#"
        {
          "velogw": "Velocloud Gateway.missing",
          "c7200": "Cisco IOS 7206VXR (Dynamips).missing",
          "vqfxre": "Juniper vQFX RE",
          "iol": "Cisco IOL",
          "vios": "Cisco vIOS Router",
          "vpcs": "Virtual PC (VPCS)",
          "c3725": "Cisco IOS 3725 (Dynamips)",
          "veos": "Arista vEOS Switch",
          "vsrx": "Juniper-2D-VSRX-S.svg.missing"
        }
        "#;

        let templates: NodeTemplates = serde_json::from_str(json).unwrap();

        assert_eq!(templates.len(), 9);

        assert_eq!(
            templates.get("c7200"),
            Some(&"Cisco IOS 7206VXR (Dynamips).missing".to_string())
        );
        assert_eq!(templates.get("iol"), Some(&"Cisco IOL".to_string()));
    }

    #[test]
    fn deserialize_node_template() {
        let json = r#"
        {
          "description": "Virtual PC (VPCS)",
          "options": {
            "config": {
              "list": ["None", "Exported"],
              "name": "Startup configuration",
              "type": "list",
              "value": "0"
            },
            "delay": {
              "name": "Delay (s)",
              "type": "input",
              "value": 0
            },
            "icon": {
              "list": { "Router.png": "Router.png", "Switch.png": "Switch.png" },
              "name": "Icon",
              "type": "list",
              "value": "PC-2D-Desktop-Generic-S.svg"
            },
            "name": {
              "name": "Name/prefix",
              "type": "input",
              "value": "VPC"
            }
          },
          "type": "vpcs"
        }
        "#;

        let template: NodeTemplate = serde_json::from_str(json).unwrap();
        assert_eq!(template.description, "Virtual PC (VPCS)");
        assert_eq!(template.node_type, "vpcs");
        assert_eq!(template.options.len(), 4);

        let config = &template.options["config"];
        assert_eq!(config["name"], "Startup configuration");
        assert_eq!(config["type"], "list");
        assert_eq!(config["value"], "0");
        assert_eq!(config["list"], serde_json::json!(["None", "Exported"]));

        let delay = &template.options["delay"];
        assert_eq!(delay["name"], "Delay (s)");
        assert_eq!(delay["type"], "input");
        assert_eq!(delay["value"], 0);
        assert!(delay.get("list").is_none());

        let icon = &template.options["icon"];
        assert_eq!(icon["name"], "Icon");
        assert_eq!(icon["value"], "PC-2D-Desktop-Generic-S.svg");
        assert!(icon["list"].is_object());

        let name = &template.options["name"];
        assert_eq!(name["name"], "Name/prefix");
        assert_eq!(name["value"], "VPC");
    }

    #[test]
    fn test_user_roles() {
        let json = r#"
        {
          "admin": "Administrator"
        }
        "#;

        let user_roles: UserRoles = serde_json::from_str(json).unwrap();
        assert_eq!(user_roles.len(), 1);
        assert_eq!(user_roles.get("admin"), Some(&"Administrator".to_string()));
    }
}
