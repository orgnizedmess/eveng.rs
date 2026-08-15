mod common;

use eveng::Result;
use eveng::folders::FolderEntry;
use eveng::labs::{CreateLabRequest, EditLabRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let labs = client.folder("/").labs();

    // Add
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
    lab.edit(&EditLabRequest {
        author: None,
        body: None,
        description: Some("Test description".to_string()),
        version: None,
        scripttimeout: None,
    })
    .await?;

    // Rename
    let lab = lab.rename("Test2").await?;

    // Move
    let folder = client
        .folders()
        .add(&FolderEntry {
            name: "New Folder".to_string(),
            path: "/".to_string(),
        })
        .await?;
    let lab = lab.move_to("/New Folder").await?;

    // After
    let resp = lab.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    lab.delete().await?;
    folder.delete().await?;

    client.logout().await?;
    Ok(())
}
