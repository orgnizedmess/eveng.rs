use eveng::labs::{CreateLabRequest, EditLabRequest, Lab};
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

    let path = "Test1.unl".to_string();

    let _resp = client
        .add_lab(&CreateLabRequest {
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
    let resp: Lab = client.lab(&path).await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    client
        .edit_lab(
            &path,
            &EditLabRequest {
                name: Some("Test2".to_string()),
                author: None,
                body: None,
                description: None,
                version: None,
                scripttimeout: None,
            },
        )
        .await?;

    let path = "Test2.unl".to_string();

    // After
    let _resp: Lab = client.lab(&path).await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Move
    // let resp = client.add_folder(&FolderEntry {
    //     name: "New Folder".to_string(),
    //     path: "/".to_string(),
    // }).await?;
    // client.move_lab(&path, "//New Folder").await?;

    // let path = "/New Folder/Test2.unl".to_string();

    // Delete
    client.delete_lab(&path).await?;
    // client.delete_folder("New Folder").await?;

    client.logout().await?;
    Ok(())
}
