mod common;

use eveng::Result;
use eveng::folders::FolderEntry;

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let root = client.folder("/")?;

    // List root folder
    let resp = root.list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Add
    let folder1 = client
        .folders()
        .add(&FolderEntry {
            name: "New Folder".to_string(),
            path: "/".to_string(),
        })
        .await?;

    let folder2 = client
        .folders()
        .add(&FolderEntry {
            name: "Test Folder".to_string(),
            path: "/".to_string(),
        })
        .await?;

    // Before
    let resp = root.list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    let folder1 = folder1.rename("New Folder 1").await?;
    folder1.move_to("/Test Folder").await?;

    // After
    let resp = folder2.list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    folder2.delete().await?;

    client.logout().await?;
    Ok(())
}
