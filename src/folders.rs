//! Endpoints for managing folders on the host.

use crate::labs::{Lab, Labs};
use crate::utils::validate_name;
use crate::{Client, Error, Result};
use serde::{Deserialize, Serialize};

/// Endpoints for managing folders on the host.
pub struct Folders {
    client: Client,
}

impl Folders {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn add(&self, params: &FolderEntry) -> Result<Folder> {
        validate_name(&params.name)?;

        self.client
            .post::<(), FolderEntry>("folders", params)
            .await?;

        let path = FolderPath::from_parts(&params.path, &params.name);
        Ok(Folder {
            client: self.client.clone(),
            path,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderListing {
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

/// Endpoints for managing a specific folder.
pub struct Folder {
    client: Client,
    path: FolderPath,
}

impl Folder {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: FolderPath::new(path),
        }
    }

    pub async fn list(&self) -> Result<FolderListing> {
        self.client
            .get(&format!("folders{}", self.path))
            .await?
            .into_data()
    }

    pub async fn rename(self, name: &str) -> Result<Folder> {
        validate_name(name)?;

        let new_path = FolderPath::from_parts(self.path.parent(), name);
        self.edit(new_path.as_str()).await?;

        Ok(Folder {
            client: self.client.clone(),
            path: new_path,
        })
    }

    pub async fn move_to(self, path: &str) -> Result<Folder> {
        let new_path = FolderPath::from_parts(path, self.path.leaf());
        self.edit(new_path.as_str()).await?;

        Ok(Folder {
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

    pub async fn delete(self) -> Result<()> {
        self.client
            .delete::<()>(&format!("folders{}", self.path))
            .await?;
        Ok(())
    }

    pub fn labs(&self) -> Labs {
        Labs::new(self.client.clone(), &self.path.as_str())
    }

    pub fn lab(&self, name: &str) -> Lab {
        eprintln!("{} {}", self.path.as_str(), name);
        Lab::new(self.client.clone(), &self.path.as_str(), name)
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
