mod common;

use eveng::Result;
use eveng::folders::{Folder, FolderEntry};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;

    // List root folder
    let resp: Folder = client.folder("").await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    let name = "New Folder".to_string();

    // Add
    let _resp = client
        .add_folder(FolderEntry {
            name: name.clone(),
            path: "/".to_string(),
        })
        .await?;

    // Before
    let resp: Folder = client.folder("").await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    client.edit_folder(&name, "/Test Folder").await?;

    let name = "Test Folder".to_string();

    // After
    let resp: Folder = client.folder("").await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    client.delete_folder(&name).await?;

    client.logout().await?;
    Ok(())
}
