//! Clients and models for managing labs within a folder.

use crate::folders::FolderPath;
use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::NodeStatus;
use crate::nodes::{NodeClient, NodesClient};
use crate::system::SystemClient;
use crate::utils::validate_name;
use crate::utils::{empty_string_is_none, map_or_seq, number_from_string};
use crate::{Client, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
    path: FolderPath,
}

impl LabsClient {
    pub(crate) fn new(client: Client, path: FolderPath) -> Self {
        Self { client, path }
    }

    /// Creates a new lab.
    pub async fn add(&self, params: AddLabRequest) -> Result<LabClient> {
        let params = params.path(self.path.as_str());

        self.client
            .post::<(), AddLabRequest>("labs", &params)
            .await?;

        // name is already validated in params, hence not validating again
        let new_path = LabPath::from_validated(self.path.clone(), &params.name);
        Ok(LabClient::from_path(self.client.clone(), new_path))
    }

    /// Returns a client for the currently open lab.
    pub async fn current(&self) -> Result<Option<LabClient>> {
        let Some(current_lab) = SystemClient::new(self.client.clone())
            .auth_status()
            .await?
            .lab
        else {
            return Ok(None);
        };

        let path = LabPath::from_str(&current_lab);
        Ok(Some(LabClient::from_path(self.client.clone(), path)))
    }
}

/// Newtype to manage the path to a lab.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LabPath(Arc<str>);

impl LabPath {
    pub fn new(path: FolderPath, name: impl AsRef<str>) -> Result<Self> {
        let name = name.as_ref();
        Self::validate(name)?;

        Ok(Self(Arc::from(Self::join(path, name))))
    }

    fn validate(name: &str) -> Result<()> {
        if name == "" {
            return Err(Error::Lab("Lab name cannot be empty".to_string()));
        }

        if !validate_name(&name, &['-', '_', ' ']) {
            return Err(Error::Lab(format!(
                "Invalid lab name '{}', must only contain letters, digits, spaces and '-'/'_'",
                name
            )));
        }

        Ok(())
    }

    pub(crate) fn from_str(path: &str) -> Self {
        Self(Arc::from(path))
    }

    pub(crate) fn from_validated(path: FolderPath, name: &str) -> Self {
        Self::from_str(&Self::join(path, name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn folder(&self) -> FolderPath {
        FolderPath::from_str(
            self.0
                .rsplit_once("/")
                .map(|(p, _)| p)
                .filter(|p| !p.is_empty())
                .unwrap_or("/"),
        )
    }

    pub fn lab_file(&self) -> &str {
        self.0.rsplit("/").next().unwrap()
    }

    pub fn lab_name(&self) -> &str {
        self.lab_file()
            .split_once(".")
            .map(|(name, _)| name)
            .unwrap()
    }

    fn join(folder: FolderPath, lab: &str) -> String {
        format!("{}/{}.unl", folder.as_str(), lab)
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
    pub(crate) fn new(client: Client, path: FolderPath, name: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            client,
            path: LabPath::new(path, name)?,
        })
    }

    pub(crate) fn from_path(client: Client, path: LabPath) -> Self {
        Self { client, path }
    }

    fn labs(&self) -> LabsClient {
        LabsClient::new(self.client.clone(), self.path.folder())
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
        self.client
            .put::<(), EditLabRequest>(&format!("labs{}", self.path), &params)
            .await?;
        Ok(())
    }

    /// Renames the lab.
    ///
    /// To update other lab details, see [`edit`](Self::edit).
    pub async fn rename(self, name: impl AsRef<str>) -> Result<Self> {
        let new_path = LabPath::new(self.path.folder(), name.as_ref())?;

        let params = EditLabRequest::new().name(new_path.lab_name());
        self.client
            .put::<(), EditLabRequest>(&format!("labs{}", self.path), &params)
            .await?;

        Ok(Self::from_path(self.client.clone(), new_path))
    }

    /// Moves the lab to the specified path.
    pub async fn move_to(self, path: impl AsRef<str>) -> Result<Self> {
        let folder = FolderPath::new(path.as_ref())?;
        let params = EditLabRequest::new().path(folder.as_str());

        self.client
            .put::<(), EditLabRequest>(&format!("labs{}/move", self.path), &params)
            .await?;

        let new_path = LabPath::from_validated(folder, self.path.lab_file());
        Ok(Self::from_path(self.client.clone(), new_path))
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

    /// Opens the lab if it isn't already open.
    ///
    /// If another lab is open, then it attempts to close that lab before
    /// opening this lab.
    pub(crate) async fn open(&self) -> Result<()> {
        match self.labs().current().await? {
            Some(lab) if lab.path == self.path => return Ok(()),
            Some(lab) => {
                return Err(Error::Lab(format!(
                    "Cannot open lab '{}' because lab '{}' is currently open.",
                    self.path, lab.path
                )));
            }
            None => {}
        }

        self.topology().await?;
        Ok(())
    }

    /// Closes the lab if it isn't already closed.
    pub async fn close(&self) -> Result<()> {
        match self.labs().current().await? {
            Some(lab) if lab.path == self.path => {}
            _ => return Ok(()),
        }

        let has_running_nodes = self
            .nodes()
            .list()
            .await?
            .iter()
            .any(|(_, v)| v.status != NodeStatus::Stopped);

        if has_running_nodes {
            return Err(Error::Lab(format!(
                "Lab '{}' cannot be closed as it has running nodes.",
                self.path
            )));
        }

        self.close_inner().await
    }

    async fn close_inner(&self) -> Result<()> {
        self.client.delete::<()>("labs/close").await?;
        Ok(())
    }

    /// Returns a client to manage nodes.
    pub fn nodes(&self) -> NodesClient {
        NodesClient::new(self.client.clone(), self.path.clone())
    }

    /// Returns a client to manage a single node.
    pub fn node(&self, id: u32) -> NodeClient {
        NodeClient::new(self.client.clone(), self.path.clone(), id)
    }

    /// Returns a client to manage networks.
    pub fn networks(&self) -> NetworksClient {
        NetworksClient::new(self.client.clone(), self.path.clone())
    }

    /// Returns a client to manage a single network.
    pub fn network(&self, id: u32) -> NetworkClient {
        NetworkClient::new(self.client.clone(), self.path.clone(), id)
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
    ///
    /// `name` must only contain letters, digits, spaces, `-` and `_`.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        LabPath::validate(&name)?;

        Ok(Self {
            name,
            scripttimeout: Some(600),
            version: Some(1),
            ..Default::default()
        })
    }

    /// Sets the name of the lab's author.
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

    /// Sets the name of the lab's author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Clears the name of the lab's author.
    pub fn clear_author(mut self) -> Self {
        self.author = Some(String::new());
        self
    }

    /// Sets the lab's usage text.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Clears the lab's usage text.
    pub fn clear_body(mut self) -> Self {
        self.body = Some(String::new());
        self
    }

    /// Sets the lab's description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Clears the lab's description.
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

    fn test_folder() -> Result<FolderPath> {
        FolderPath::new("/Test Folder")
    }

    fn new_folder() -> Result<FolderPath> {
        FolderPath::new("/New Folder")
    }

    #[test]
    fn valid_lab_path() -> Result<()> {
        let path = LabPath::new(test_folder()?, "Test")?;

        assert_eq!(path.as_str(), "/Test Folder/Test.unl");
        assert_eq!(path.folder().as_str(), "/Test Folder");
        assert_eq!(path.lab_file(), "Test.unl");

        Ok(())
    }

    #[test]
    fn invalid_lab_path() -> Result<()> {
        let path = LabPath::new(test_folder()?, "Lab: Test");
        assert!(path.is_err());

        Ok(())
    }

    #[test]
    fn lab_rename() -> Result<()> {
        let path = LabPath::new(test_folder()?, "Test")?;
        let new_path = LabPath::new(path.folder(), "Test1")?;

        assert_eq!(new_path.as_str(), "/Test Folder/Test1.unl");
        assert_eq!(new_path.folder().as_str(), "/Test Folder");
        assert_eq!(new_path.lab_file(), "Test1.unl");
        assert_eq!(new_path.lab_name(), "Test1");

        Ok(())
    }

    #[test]
    fn lab_move() -> Result<()> {
        let path = LabPath::new(test_folder()?, "Test")?;
        let new_path = LabPath::new(new_folder()?, path.lab_name())?;

        assert_eq!(new_path.as_str(), "/New Folder/Test.unl");
        assert_eq!(new_path.folder().as_str(), "/New Folder");
        assert_eq!(new_path.lab_file(), "Test.unl");

        Ok(())
    }
}
