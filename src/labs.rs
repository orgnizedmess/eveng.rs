use crate::utils::number_from_string;
use crate::{Client, Result};
use serde::{Deserialize, Serialize};

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
    pub name: Option<String>,
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
pub struct MoveLabRequest {
    pub dest_path: String,
}

impl Client {
    pub async fn lab(&self, path: &str) -> Result<Lab> {
        self.get(&format!("labs/{}", path)).await?.into_data()
    }

    pub async fn add_lab(&self, params: CreateLabRequest) -> Result<()> {
        self.post::<(), CreateLabRequest>("labs", params).await?;
        Ok(())
    }

    pub async fn edit_lab(&self, path: &str, params: EditLabRequest) -> Result<()> {
        self.put::<(), EditLabRequest>(&format!("labs/{}", path), params)
            .await?;
        Ok(())
    }

    // Doesn't run correctly via API, works if I make files via GUI
    pub async fn move_lab(&self, src_path: &str, dest_path: &str) -> Result<()> {
        self.put::<(), MoveLabRequest>(
            &format!("labs/{}/move", src_path),
            MoveLabRequest {
                dest_path: dest_path.to_string(),
            },
        )
        .await?;
        Ok(())
    }

    pub async fn delete_lab(&self, path: &str) -> Result<()> {
        self.delete::<()>(&format!("labs/{}", path)).await?;
        Ok(())
    }

    // Visible on GUI, undocumented in API
    // Yes, the endpoint is with a capital L
    pub async fn lock_lab(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/Lock", self.lab_path))
            .await?;
        Ok(())
    }

    // Visible on GUI, undocumented in API
    pub async fn unlock_lab(&self) -> Result<()> {
        self.get::<()>(&format!("labs/{}/Unlock", self.lab_path))
            .await?;
        Ok(())
    }
}
