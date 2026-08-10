use crate::labs::{Lab, Labs};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
// use crate::utils::UrlPath;

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
    pub path: String,
    // Unix timestamp
    pub umtime: u64,
    pub mtime: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditFolderRequest {
    pub path: String,
}

pub struct Folders {
    client: Client,
}

impl Folders {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn add(&self, params: &FolderEntry) -> Result<Folder> {
        self.client
            .post::<(), FolderEntry>("folders", params)
            .await?;

        let path = format!("{}{}", params.path, params.name);
        Ok(Folder::new(
            self.client.clone(),
            &path.trim_start_matches("/"),
        ))
    }
}

pub struct Folder {
    client: Client,
    path: String,
}

impl Folder {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    pub async fn list(&self) -> Result<FolderListing> {
        self.client
            .get(&format!("folders/{}", self.path))
            .await?
            .into_data()
    }

    // TODO: Make two separate endpoints for move and rename
    pub async fn edit(&mut self, dest_path: &str) -> Result<()> {
        self.client
            .put::<(), EditFolderRequest>(
                &format!("folders/{}", self.path),
                &EditFolderRequest {
                    path: dest_path.to_string(),
                },
            )
            .await?;
        self.path = dest_path.trim_start_matches("/").to_string();
        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        self.client
            .delete::<()>(&format!("folders/{}", self.path))
            .await?;
        Ok(())
    }

    pub fn labs(&self) -> Labs {
        Labs::new(self.client.clone(), &self.path)
    }

    pub fn lab(&self, name: &str) -> Lab {
        Lab::new(self.client.clone(), &format!("{}{}", &self.path, name))
    }
}
