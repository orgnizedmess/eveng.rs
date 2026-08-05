use eveng::folders::{Folder, FolderEntry};
use eveng::{Client, Result};

fn test_client() -> Client {
    Client::new(
        "http://192.168.0.141".to_string(),
        "admin".to_string(),
        "eve".to_string(),
        "Test.unl".to_string(),
    )
    .unwrap()
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = test_client();
    client.login().await.unwrap();

    // List root folder
    let resp: Folder = client.folder("").await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    let name = "New Folder".to_string();

    // Add
    let _resp = client
        .add_folder(&FolderEntry {
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
