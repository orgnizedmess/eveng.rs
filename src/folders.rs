//! Clients and models for managing folders on the EVE-NG instance.

use crate::labs::{LabClient, LabsClient};
use crate::utils::validate_name;
use crate::{Client, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A client for managing folders.
pub struct FoldersClient {
    client: Client,
}

impl FoldersClient {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Creates a new folder.
    pub async fn add(&self, params: &FolderEntry) -> Result<FolderClient> {
        let path = FolderPath::from_parts(&params.path, &params.name)?;

        self.client
            .post::<(), FolderEntry>("folders", params)
            .await?;

        Ok(FolderClient {
            client: self.client.clone(),
            path,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub folders: Vec<FolderEntry>,
    pub labs: Vec<LabEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabEntry {
    #[serde(rename = "file")]
    pub filename: String,
    /// Modification time
    pub mtime: String,
    pub path: String,
    /// Modification time as a unix timestamp
    pub umtime: u64,
}

/// Newtype to manage the path to a folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FolderPath(Arc<str>);

impl std::fmt::Display for FolderPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FolderPath {
    pub(crate) fn new(path: impl AsRef<str>) -> Result<Self> {
        let path = path.as_ref();
        Self::validate(path)?;

        Ok(Self(Arc::from(path)))
    }

    fn validate(path: &str) -> Result<()> {
        if path == "" {
            return Err(Error::Folder("Path cannot be empty".to_string()));
        }

        if !path.starts_with("/") {
            return Err(Error::Folder("Path must be an absolute path".to_string()));
        }

        for segment in path.split("/") {
            Self::validate_segment(segment)?;
        }

        Ok(())
    }

    fn validate_segment(name: &str) -> Result<()> {
        if !validate_name(&name, &['-', '_', ' ']) {
            return Err(Error::Folder(format!(
                "Invalid folder segment '{}', must only contain letters, digits, spaces and '-'/'_'",
                name
            )));
        }
        Ok(())
    }

    // Creating from segments rather than the full path
    pub(crate) fn from_parts(parent: impl AsRef<str>, leaf: impl AsRef<str>) -> Result<Self> {
        Self::new(Self::join(parent.as_ref(), leaf.as_ref()))
    }

    // For cases where the path is already valid (eg: from an API response)
    pub(crate) fn from_str(path: &str) -> Self {
        Self(Arc::from(path))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn parent(&self) -> &str {
        self.0
            .rsplit_once("/")
            .map(|(p, _)| p)
            .filter(|p| !p.is_empty())
            .unwrap_or("/")
    }

    pub(crate) fn leaf(&self) -> &str {
        self.0.rsplit("/").next().unwrap()
    }

    // Only leaf needs to be validated
    pub(crate) fn rename(&self, name: &str) -> Result<Self> {
        Self::validate_segment(name)?;
        Ok(Self::from_str(&Self::join(self.parent(), name)))
    }

    // Only parent needs to be validated
    pub(crate) fn move_to(&self, path: &str) -> Result<Self> {
        Self::validate(path)?;
        Ok(Self::from_str(&Self::join(path, self.leaf())))
    }

    fn join(parent: &str, leaf: &str) -> String {
        format!("{}/{}", parent.trim_end_matches("/"), leaf)
    }
}

/// A client for managing a single folder.
pub struct FolderClient {
    client: Client,
    path: FolderPath,
}

impl FolderClient {
    pub(crate) fn new(client: Client, path: &str) -> Result<Self> {
        Ok(Self {
            client,
            path: FolderPath::new(path)?,
        })
    }

    // Lists the contents of the folder.
    pub async fn list(&self) -> Result<Folder> {
        self.client
            .get(&format!("folders{}", self.path))
            .await?
            .into_data()
    }

    // Renames the folder.
    pub async fn rename(self, name: impl AsRef<str>) -> Result<FolderClient> {
        let name = name.as_ref();

        let new_path = self.path.rename(name)?;
        self.edit(new_path.as_str()).await?;

        Ok(FolderClient {
            client: self.client.clone(),
            path: new_path,
        })
    }

    /// Moves the folder to the specified path.
    pub async fn move_to(self, path: impl AsRef<str>) -> Result<FolderClient> {
        let path = path.as_ref();

        let new_path = self.path.move_to(path)?;
        self.edit(new_path.as_str()).await?;

        Ok(FolderClient {
            client: self.client.clone(),
            path: new_path,
        })
    }

    async fn edit(&self, path: &str) -> Result<()> {
        let params = serde_json::json!({path: path.to_string()});

        self.client
            .put::<(), serde_json::Value>(&format!("folders{}", self.path), &params)
            .await?;

        Ok(())
    }

    /// Deletes the folder.
    pub async fn delete(self) -> Result<()> {
        self.client
            .delete::<()>(&format!("folders{}", self.path))
            .await?;
        Ok(())
    }

    /// Returns a client for managing labs.
    pub fn labs(&self) -> LabsClient {
        LabsClient::new(self.client.clone(), self.path.clone())
    }

    /// Returns a client for managing a single lab.
    pub fn lab(&self, name: &str) -> Result<LabClient> {
        LabClient::new(self.client.clone(), self.path.clone(), name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_folder_path() -> Result<()> {
        let path = FolderPath::new("/New Folder")?;

        assert_eq!(path.parent(), "/");
        assert_eq!(path.leaf(), "New Folder");

        Ok(())
    }

    #[test]
    fn invalid_folder_path() -> Result<()> {
        let path = FolderPath::new("New Folder");
        assert!(path.is_err());

        let path = FolderPath::new("/New+Folder");
        assert!(path.is_err());
        Ok(())
    }

    #[test]
    fn folder_rename() -> Result<()> {
        let path = FolderPath::new("/New Folder")?;
        let new_path = path.rename("Test Folder")?;

        assert_eq!(new_path.parent(), "/");
        assert_eq!(new_path.leaf(), "Test Folder");

        Ok(())
    }

    #[test]
    fn folder_move() -> Result<()> {
        let path = FolderPath::new("/New Folder")?;
        let new_path = path.move_to("/Test Folder")?;

        assert_eq!(new_path.as_str(), "/Test Folder/New Folder");
        assert_eq!(new_path.parent(), "/Test Folder");
        assert_eq!(new_path.leaf(), "New Folder");

        Ok(())
    }
}
