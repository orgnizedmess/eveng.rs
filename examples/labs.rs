mod common;

use eveng::Result;
use eveng::labs::{CreateLabRequest, EditLabRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let labs = client.folder("/").labs();

    let lab = labs
        .add(&CreateLabRequest {
            name: "Test1".to_string(),
            path: "/".to_string(),
            version: 1,
            scripttimeout: None,
            author: None,
            body: None,
            description: None,
        })
        .await?;

    // Before
    let resp = lab.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    // TODO: Doesn't return updated name for now, so the get below fails
    // lab.edit(&EditLabRequest {
    //     name: Some("Test2".to_string()),
    //     author: None,
    //     body: None,
    //     description: None,
    //     version: None,
    //     scripttimeout: None,
    // })
    // .await?;

    // // After
    // let resp = lab.get().await?;
    // eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Move
    // let resp = labs.add_folder(&FolderEntry {
    //     name: "New Folder".to_string(),
    //     path: "/".to_string(),
    // }).await?;
    // labs.move_lab(&path, "//New Folder").await?;

    // let path = "/New Folder/Test2.unl".to_string();

    // Delete
    lab.delete().await?;
    // client.delete_folder("New Folder").await?;

    client.logout().await?;
    Ok(())
}
