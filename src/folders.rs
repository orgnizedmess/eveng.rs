use crate::{Client, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FolderEntry {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub file: String,
    pub path: String,
    // Unix timestamp
    pub umtime: u64,
    pub mtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub folders: Vec<FolderEntry>,
    pub labs: Vec<FileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditFolderRequest {
    pub path: String,
}

impl Client {
    pub async fn folder(&self, name: &str) -> Result<Folder> {
        self.get(&format!("folders/{}", name)).await?.into_data()
    }

    pub async fn add_folder(&self, params: FolderEntry) -> Result<()> {
        self.post::<(), FolderEntry>("folders", params).await?;
        Ok(())
    }

    // Move and/or rename
    pub async fn edit_folder(&self, src_path: &str, dest_path: &str) -> Result<()> {
        self.put::<(), EditFolderRequest>(
            &format!("folders/{}", src_path),
            EditFolderRequest {
                path: dest_path.to_string(),
            },
        )
        .await?;
        Ok(())
    }

    pub async fn delete_folder(&self, path: &str) -> Result<()> {
        self.delete::<()>(&format!("folders/{}", path)).await?;
        Ok(())
    }
}
