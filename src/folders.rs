//! Clients and models for managing folders on the EVE-NG instance.

use crate::labs::{LabClient, LabsClient};
use crate::utils::validate_name;
use crate::{Client, Result};
use serde::{Deserialize, Serialize};

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
        validate_name(&params.name)?;

        self.client
            .post::<(), FolderEntry>("folders", params)
            .await?;

        let path = FolderPath::from_parts(&params.path, &params.name);
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

#[derive(Debug)]
pub struct FolderPath(String);

impl std::fmt::Display for FolderPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl FolderPath {
    pub fn new(path: &str) -> Self {
        Self(path.to_string())
    }

    pub fn from_parts(parent: &str, leaf: &str) -> Self {
        Self(format!("{}/{}", parent.trim_end_matches("/"), leaf))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parent(&self) -> &str {
        self.0
            .rsplit_once("/")
            .map(|(p, _)| p)
            .filter(|p| !p.is_empty())
            .unwrap_or("/")
    }

    pub fn leaf(&self) -> &str {
        self.0.rsplit("/").next().unwrap()
    }
}

/// A client for managing a single folder.
pub struct FolderClient {
    client: Client,
    path: FolderPath,
}

impl FolderClient {
    pub(crate) fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: FolderPath::new(path),
        }
    }

    // Lists the contents of the folder.
    pub async fn list(&self) -> Result<Folder> {
        self.client
            .get(&format!("folders{}", self.path))
            .await?
            .into_data()
    }

    // Renames the folder.
    pub async fn rename(self, name: &str) -> Result<FolderClient> {
        validate_name(name)?;

        let new_path = FolderPath::from_parts(self.path.parent(), name);
        self.edit(new_path.as_str()).await?;

        Ok(FolderClient {
            client: self.client.clone(),
            path: new_path,
        })
    }

    /// Moves the folder to the specified path.
    pub async fn move_to(self, folder_path: &str) -> Result<FolderClient> {
        let new_path = FolderPath::from_parts(folder_path, self.path.leaf());
        self.edit(new_path.as_str()).await?;

        Ok(FolderClient {
            client: self.client.clone(),
            path: new_path,
        })
    }

    async fn edit(&self, path: &str) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct EditFolderRequest {
            path: String,
        }

        let params = &EditFolderRequest {
            path: path.to_string(),
        };

        self.client
            .put::<(), EditFolderRequest>(&format!("folders{}", self.path), params)
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
        LabsClient::new(self.client.clone(), self.path.as_str())
    }

    /// Returns a client for managing a single lab.
    pub fn lab(&self, name: &str) -> LabClient {
        LabClient::new(self.client.clone(), self.path.as_str(), name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folder_path() {
        let path = FolderPath("/New Folder".to_string());

        assert_eq!(path.parent(), "/");
        assert_eq!(path.leaf(), "New Folder");
    }

    #[test]
    fn folder_rename() {
        let path = FolderPath::new("/New Folder");
        let new_path = FolderPath::from_parts(path.parent(), "Test Folder");

        assert_eq!(new_path.parent(), "/");
        assert_eq!(new_path.leaf(), "Test Folder");
    }

    #[test]
    fn folder_move() {
        let path = FolderPath::new("/New Folder");
        let new_path = FolderPath::from_parts("/Test Folder", path.leaf());

        assert_eq!(new_path.as_str(), "/Test Folder/New Folder");
        assert_eq!(new_path.parent(), "/Test Folder");
        assert_eq!(new_path.leaf(), "New Folder");
    }
}
