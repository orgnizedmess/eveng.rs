mod common;

use eveng::Result;
use eveng::folders::FolderEntry;
use eveng::labs::{AddLabRequest, EditLabRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let folder = client.folder("/");

    // Add
    let req = AddLabRequest::new("Test1")?;
    let lab = folder.labs().add(req).await?;

    // Before
    let resp = lab.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    lab.edit(EditLabRequest::new().description("Test description"))
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
