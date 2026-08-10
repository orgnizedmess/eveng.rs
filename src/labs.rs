use crate::networks::{Network, Networks};
use crate::nodes::{Node, Nodes};
use crate::utils::{map_or_empty_seq, nested_map_or_empty_seq, number_from_string};
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct LabInfo {
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

pub struct Labs {
    client: Client,
    path: String,
}

impl Labs {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    pub async fn add(&self, params: &CreateLabRequest) -> Result<Lab> {
        self.client
            .post::<(), CreateLabRequest>("labs", params)
            .await?;
        Ok(Lab::new(
            self.client.clone(),
            &format!("{}{}.unl", &self.path, &params.name),
        ))
    }
}

pub struct Lab {
    client: Client,
    path: String,
}

impl Lab {
    pub fn new(client: Client, path: &str) -> Self {
        Self {
            client,
            path: path.to_string(),
        }
    }

    pub async fn get(&self) -> Result<LabInfo> {
        self.client
            .get(&format!("labs/{}", self.path))
            .await?
            .into_data()
    }

    pub async fn edit(&mut self, params: &EditLabRequest) -> Result<()> {
        self.client
            .put::<(), EditLabRequest>(&format!("labs/{}", self.path), params)
            .await?;
        // TODO: Update path if name is changed
        // self.path contains previous name here, so below would result in an incorrect path
        // Lab path type should help here
        // if let Some(name) = &params.name {
        //    self.path = format!("{}{}.unl", self.path.trim_start_matches("/"), name);
        // }
        Ok(())
    }

    // Doesn't run correctly via API, works if I make files via GUI
    pub async fn move_to_folder(&self, dest_path: &str) -> Result<()> {
        self.client
            .put::<(), MoveLabRequest>(
                &format!("labs/{}/move", self.path),
                &MoveLabRequest {
                    dest_path: dest_path.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    pub async fn delete(&self) -> Result<()> {
        self.client
            .delete::<()>(&format!("labs/{}", self.path))
            .await?;
        Ok(())
    }

    // Visible on GUI, undocumented in API
    // Yes, the endpoint is with a capital L
    pub async fn lock(&self) -> Result<()> {
        self.client
            .put::<(), ()>(&format!("labs/{}/Lock", self.path), &())
            .await?;
        Ok(())
    }

    // Visible on GUI, undocumented in API
    pub async fn unlock(&self) -> Result<()> {
        self.client
            .put::<(), ()>(&format!("labs/{}/Unlock", self.path), &())
            .await?;
        Ok(())
    }

    pub fn nodes(&self) -> Nodes {
        Nodes::new(self.client.clone(), &self.path)
    }

    pub fn node(&self, id: i32) -> Node {
        Node::new(self.client.clone(), &self.path, id)
    }

    pub fn networks(&self) -> Networks {
        Networks::new(self.client.clone(), &self.path)
    }

    pub fn network(&self, id: i32) -> Network {
        Network::new(self.client.clone(), &self.path, id)
    }

    pub async fn topology(&self) -> Result<Vec<TopologyEntry>> {
        self.client
            .get(&format!("labs/{}/topology", self.path))
            .await?
            .into_data()
    }

    // Part of labs API in source code
    pub async fn links(&self) -> Result<Links> {
        self.client
            .get(&format!("labs/{}/links", self.path))
            .await?
            .into_data()
    }
}
