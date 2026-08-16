//! Clients and models for managing labs within a folder.

use crate::networks::{NetworkClient, NetworksClient};
use crate::nodes::{NodeClient, NodesClient};
use crate::utils::validate_name;
use crate::utils::{map_or_empty_seq, nested_map_or_empty_seq, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Lab {
    pub author: String,
    pub body: String,
    pub description: String,
    // without extension, with extension returns an error
    pub filename: String,
    // uuid style ids
    pub id: Option<String>,
    // Mentioned in source code but not in API docs, value is 0 or 1
    #[serde(deserialize_with = "number_from_string")]
    pub lock: i32,
    pub name: String,
    pub scripttimeout: i32,
    #[serde(deserialize_with = "number_from_string")]
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLabRequest {
    pub name: String,
    pub path: String,
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripttimeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

// From the source code:
// If an attribute is set and is valid, then it will be used
// If an attribute is not set, then the original is maintained.
// If an attribute is set and empty, then the current one is deleted.
//
// Seems kind of important, because then I have to make the distinction between
// None and empty values clear via the API design or docs.
// But also, would this only apply to strings?
#[derive(Debug, Serialize, Deserialize)]
pub struct EditLabRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripttimeout: Option<i32>,
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
    #[serde(deserialize_with = "map_or_empty_seq")]
    pub ethernet: HashMap<i32, String>,
    #[serde(deserialize_with = "nested_map_or_empty_seq")]
    pub serial: HashMap<i32, HashMap<i32, String>>,
}

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
    pub async fn add(&self, params: &CreateLabRequest) -> Result<LabClient> {
        validate_name(&params.name)?;

        self.client
            .post::<(), CreateLabRequest>("labs", params)
            .await?;

        Ok(LabClient::new(
            self.client.clone(),
            &self.path,
            &params.name,
        ))
    }
}

#[derive(Debug)]
pub struct LabPath(String);

impl LabPath {
    pub fn new(path: &str, name: &str) -> Self {
        let mut lab = name.to_string();

        if !lab.ends_with(".unl") {
            lab.push_str(".unl")
        }

        Self(format!("{}/{}", path.trim_end_matches("/"), lab))
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

pub struct LabClient {
    client: Client,
    path: LabPath,
}

impl LabClient {
    pub(crate) fn new(client: Client, path: &str, name: &str) -> Self {
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
    /// To change the lab's name, see [`rename`](Self::rename()).
    pub async fn edit(&self, params: &EditLabRequest) -> Result<()> {
        // Docs specify that only one parameter can be changed per request, but
        // editing multiple parameters works?
        self.client
            .put::<(), EditLabRequest>(&format!("labs{}", self.path), params)
            .await?;
        Ok(())
    }

    /// Renames the lab.
    ///
    /// To update other lab details, see [`edit`](Self::edit).
    pub async fn rename(self, name: &str) -> Result<LabClient> {
        validate_name(name)?;

        #[derive(Debug, Serialize)]
        struct RenameLabRequest {
            name: String,
        }

        let params = &RenameLabRequest {
            name: name.to_string(),
        };
        self.client
            .put::<(), RenameLabRequest>(&format!("labs{}", self.path), params)
            .await?;

        Ok(LabClient::new(
            self.client.clone(),
            self.path.folder(),
            name,
        ))
    }

    /// Moves the lab to the specified path.
    pub async fn move_to(self, folder_path: &str) -> Result<LabClient> {
        #[derive(Debug, Serialize)]
        struct MoveLabRequest {
            path: String,
        }

        let params = &MoveLabRequest {
            path: folder_path.to_string(),
        };
        self.client
            .put::<(), MoveLabRequest>(&format!("labs{}/move", self.path), params)
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

    pub fn nodes(&self) -> NodesClient {
        NodesClient::new(self.client.clone(), self.path.as_str())
    }

    pub fn node(&self, id: i32) -> NodeClient {
        NodeClient::new(self.client.clone(), self.path.as_str(), id)
    }

    pub fn networks(&self) -> NetworksClient {
        NetworksClient::new(self.client.clone(), self.path.as_str())
    }

    pub fn network(&self, id: i32) -> NetworkClient {
        NetworkClient::new(self.client.clone(), self.path.as_str(), id)
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
