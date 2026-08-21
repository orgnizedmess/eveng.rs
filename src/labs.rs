//! Clients and models for managing labs within a folder.

use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::{NodeClient, NodesClient};
use crate::utils::validate_pathname;
use crate::utils::{empty_string_is_none, map_or_seq, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lab {
    /// Name of the lab file without the path.
    pub filename: String,

    pub id: String,

    pub lock: u8,

    /// Name of the lab file, without the path and extension.
    pub name: String,

    /// Value in seconds used for the “Configuration Export” and “Boot from
    /// exported configs” operations
    pub scripttimeout: u32,

    #[serde(deserialize_with = "number_from_string")]
    pub version: u32,

    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub author: Option<String>,

    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub body: Option<String>,

    #[serde(
        deserialize_with = "empty_string_is_none",
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopologyEntry {
    pub destination: String,
    pub destination_label: String,
    pub destination_type: String,
    pub source: String,
    pub source_label: String,
    pub source_type: String,
    #[serde(rename = "type")]
    pub topology_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Links {
    #[serde(deserialize_with = "map_or_seq")]
    pub ethernet: HashMap<i32, String>,
    #[serde(deserialize_with = "map_or_seq")]
    pub serial: HashMap<i32, HashMap<i32, String>>,
}

/// A client to manage labs.
pub struct LabsClient {
    client: Client,
    path: String,
}

impl LabsClient {
    pub(crate) fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    /// Creates a new lab.
    pub async fn add(&self, params: AddLabRequest) -> Result<LabClient> {
        let params = params.path(&self.path);

        self.client
            .post::<(), AddLabRequest>("labs", &params)
            .await?;

        Ok(LabClient::new(
            self.client.clone(),
            &params.path,
            &params.name,
        ))
    }
}

#[derive(Debug)]
pub struct LabPath(String);

impl LabPath {
    pub fn new(path: impl Into<String>, name: impl Into<String>) -> Self {
        let mut lab = name.into();

        if !lab.ends_with(".unl") {
            lab.push_str(".unl")
        }

        Self(format!("{}/{}", path.into().trim_end_matches("/"), lab))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn folder(&self) -> &str {
        self.0
            .rsplit_once("/")
            .map(|(p, _)| p)
            .filter(|p| !p.is_empty())
            .unwrap_or("/")
    }

    pub fn lab(&self) -> &str {
        self.0.rsplit("/").next().unwrap()
    }

    pub fn lab_name(&self) -> &str {
        self.lab().split_once(".").map(|(name, _)| name).unwrap()
    }
}

impl std::fmt::Display for LabPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A client to manage a single lab.
pub struct LabClient {
    client: Client,
    path: LabPath,
}

impl LabClient {
    pub(crate) fn new(client: Client, path: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            client,
            path: LabPath::new(path, name),
        }
    }

    /// Gets the lab's details.
    pub async fn get(&self) -> Result<Lab> {
        self.client
            .get(&format!("labs{}", self.path))
            .await?
            .into_data()
    }

    /// Updates the lab's details.
    ///
    /// To change the lab's name, see [`rename`](Self::rename).
    pub async fn edit(&self, params: EditLabRequest) -> Result<()> {
        // Docs specify that only one parameter can be changed per request, but
        // editing multiple parameters works?
        self.client
            .put::<(), EditLabRequest>(&format!("labs{}", self.path), &params)
            .await?;
        Ok(())
    }

    /// Renames the lab.
    ///
    /// To update other lab details, see [`edit`](Self::edit).
    pub async fn rename(self, name: impl Into<String>) -> Result<LabClient> {
        let name = name.into();
        validate_pathname(&name)?;

        let params = EditLabRequest::new().name(&name);

        self.client
            .put::<(), EditLabRequest>(&format!("labs{}", self.path), &params)
            .await?;

        Ok(LabClient::new(
            self.client.clone(),
            self.path.folder(),
            name,
        ))
    }

    /// Moves the lab to the specified path.
    pub async fn move_to(self, folder_path: &str) -> Result<LabClient> {
        let params = EditLabRequest::new().path(folder_path);

        self.client
            .put::<(), EditLabRequest>(&format!("labs{}/move", self.path), &params)
            .await?;

        Ok(LabClient::new(
            self.client.clone(),
            folder_path,
            self.path.lab_name(),
        ))
    }

    /// Deletes the lab.
    pub async fn delete(self) -> Result<()> {
        self.client
            .delete::<()>(&format!("labs{}", self.path))
            .await?;
        Ok(())
    }

    /// Locks the lab.
    pub async fn lock(&self) -> Result<()> {
        self.client
            .put::<(), ()>(&format!("labs{}/Lock", self.path), &())
            .await?;
        Ok(())
    }

    /// Unlocks the lab.
    pub async fn unlock(&self) -> Result<()> {
        self.client
            .put::<(), ()>(&format!("labs{}/Unlock", self.path), &())
            .await?;
        Ok(())
    }

    /// Lists the lab's topology.
    pub async fn topology(&self) -> Result<Vec<TopologyEntry>> {
        self.client
            .get(&format!("labs{}/topology", self.path))
            .await?
            .into_data()
    }

    /// Lists all remote endpoints for both ethernet and serial interfaces in
    /// the lab.
    pub async fn links(&self) -> Result<Links> {
        self.client
            .get(&format!("labs{}/links", self.path))
            .await?
            .into_data()
    }

    /// Returns a client to manage nodes.
    pub fn nodes(&self) -> NodesClient {
        NodesClient::new(self.client.clone(), self.path.as_str())
    }

    /// Returns a client to manage a single node.
    pub fn node(&self, id: i32) -> NodeClient {
        NodeClient::new(self.client.clone(), self.path.as_str(), id)
    }

    /// Returns a client to manage networks.
    pub fn networks(&self) -> NetworksClient {
        NetworksClient::new(self.client.clone(), self.path.as_str())
    }

    /// Returns a client to manage a single network.
    pub fn network(&self, id: i32) -> NetworkClient {
        NetworkClient::new(self.client.clone(), self.path.as_str(), id)
    }
}

/// Request for adding a lab.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AddLabRequest {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scripttimeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u32>,
}

impl AddLabRequest {
    /// Creates a new request for adding a lab.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_pathname(&name)?;

        Ok(Self {
            name,
            scripttimeout: Some(600),
            version: Some(1),
            ..Default::default()
        })
    }

    /// Sets the lab's author name.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Sets the lab's usage text.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Sets the lab's description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the lab's version.
    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the lab's script timeout.
    ///
    /// The script timeout is the value in seconds used for the “Configuration
    /// Export” and “Boot from exported configs” operations.
    pub fn scripttimeout(mut self, scripttimeout: u32) -> Self {
        self.scripttimeout = Some(scripttimeout);
        self
    }

    pub(crate) fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EditLabRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scripttimeout: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u32>,
}

impl EditLabRequest {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Sets the lab's author name.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn clear_author(mut self) -> Self {
        self.author = Some(String::new());
        self
    }

    /// Sets the lab's usage text.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn clear_body(mut self) -> Self {
        self.body = Some(String::new());
        self
    }

    /// Sets the lab's description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn clear_description(mut self) -> Self {
        self.description = Some(String::new());
        self
    }

    /// Sets the lab's version.
    pub fn scripttimeout(mut self, scripttimeout: u32) -> Self {
        self.scripttimeout = Some(scripttimeout);
        self
    }

    /// Sets the lab's script timeout.
    pub fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    pub(crate) fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub(crate) fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_lab_path() {
        let path = LabPath::new("/Test Folder", "Test");

        assert_eq!(path.as_str(), "/Test Folder/Test.unl");
        assert_eq!(path.folder(), "/Test Folder");
        assert_eq!(path.lab_name(), "Test");
    }

    #[test]
    fn test_folder_rename() {
        let path = LabPath::new("/Test Folder", "Test");
        let new_path = LabPath::new(path.folder(), "Test1");

        assert_eq!(new_path.as_str(), "/Test Folder/Test1.unl");
        assert_eq!(new_path.folder(), "/Test Folder");
        assert_eq!(new_path.lab(), "Test1.unl");
        assert_eq!(new_path.lab_name(), "Test1");
    }

    #[test]
    fn test_folder_move() {
        let path = LabPath::new("/Test Folder", "Test");
        let new_path = LabPath::new("/New Folder", path.lab());

        assert_eq!(new_path.as_str(), "/New Folder/Test.unl");
        assert_eq!(new_path.folder(), "/New Folder");
        assert_eq!(new_path.lab(), "Test.unl");
    }
}
