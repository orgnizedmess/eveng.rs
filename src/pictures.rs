// kinda broken for now
use crate::utils::number_from_string;
use crate::{Client, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Picture {
    pub height: i32,
    #[serde(deserialize_with = "number_from_string")]
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub picture_type: String,
    pub width: i32,
    pub map: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPictureRequest {
    pub name: String,
    pub file: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditPictureRequest {
    pub name: String,
    pub map: Option<String>,
    // map but for custom (outside of lab) objects?
    // always empty in the examples I've tried so far
    pub custommap: Option<String>,
}

// returns Vec when empty, how to deal with that?
pub type Pictures = HashMap<String, Picture>;

impl Client {
    pub async fn pictures(&self) -> Result<Pictures> {
        self.get(&format!("/labs/{}/pictures", self.lab_path))
            .await?
            .into_data()
    }

    pub async fn picture(&self, id: i32) -> Result<Picture> {
        self.get(&format!("/labs/{}/pictures/{}", self.lab_path, id))
            .await?
            .into_data()
    }

    // Not checking for API errors yet
    pub async fn picture_data(&self, id: i32, width: i32, height: i32) -> Result<Vec<u8>> {
        let resp = self
            .client
            .get(format!(
                "{}/api/labs/{}/pictures/{}/data/{}/{}",
                self.base_url, self.lab_path, id, width, height
            ))
            .send()
            .await?;
        Ok(resp.bytes().await?.to_vec())
    }

    // filename is the name of the file on your machine, name is what you want to name
    // it on EVE-NG
    // Untested
    pub async fn add_picture(&self, filename: &str, file_bytes: Vec<u8>, name: &str) -> Result<()> {
        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(filename.to_string());

        let form = reqwest::multipart::Form::new()
            .text("name", name.to_string())
            .part("file", part); // field name likely doesn't matter — PHP iterates all of $_FILES

        let _response = self
            .client
            .post(format!(
                "{}/api/labs/{}/pictures",
                self.base_url, self.lab_path
            ))
            .multipart(form)
            .send()
            .await?;

        Ok(())
    }

    // Editing an image map doesn't make much sense as an API call?
    // Untested
    pub async fn edit_picture(&self, id: i32, params: EditPictureRequest) -> Result<()> {
        self.put::<(), EditPictureRequest>(
            &format!("labs/{}/pictures/{}", self.lab_path, id),
            params,
        )
        .await?;
        Ok(())
    }

    // Untested
    pub async fn delete_picture(&self, id: i32) -> Result<()> {
        self.delete::<()>(&format!("labs/{}/pictures/{}", self.lab_path, id))
            .await?;
        Ok(())
    }
}
