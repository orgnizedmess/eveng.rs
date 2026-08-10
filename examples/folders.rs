mod common;

use eveng::Result;
use eveng::folders::FolderEntry;

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;

    // List root folder
    let resp = client.folder("/").list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Add
    let mut folder = client
        .folders()
        .add(&FolderEntry {
            name: "New Folder".to_string(),
            path: "/".to_string(),
        })
        .await?;

    // Before
    let resp = folder.list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    folder.edit("/Test Folder").await?;

    // After
    let resp = folder.list().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    folder.delete().await?;

    client.logout().await?;
    Ok(())
}
