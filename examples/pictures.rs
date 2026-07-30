// untested
use eveng::{Client, Result};
use eveng::pictures::{Pictures, Picture};

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

    let _path = "Test1.unl".to_string();

    let resp: Pictures = client.pictures().await?;
    eprintln!("{:?}", &resp);

    let resp: Picture = client.picture(1).await?;
    eprintln!("{:?}", &resp);
    // let data: Vec<u8> = client.picture_data(1, 100, 100).await?;
    // eprintln!("{:?}", &data);

//     let resp = client.add_picture(&CreatePictureRequest {
//             name: "Test1".to_string(),
//             path: "/".to_string(),
//             version: 1,
//             scripttimeout: None,
//             author: None,
//             body: None,
//             description: None,
//         }).await?;
//
//     // Before
//     let resp: Picture = client.picture(&path).await?;
//     eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());
//
//     // Edit
//     client
//         .edit_picture(
//             &path,
//             &EditPictureRequest {
//                 name: Some("Test2".to_string()),
//                 author: None,
//                 body: None,
//                 description: None,
//                 version: None,
//                 scripttimeout: None,
//             },
//         )
//         .await?;
//
//     let path = "Test2.unl".to_string();
//
//     // After
//     let resp: Picture = client.picture(&path).await?;
//     eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());
//
//     // Move
//     // let resp = client.add_folder(&FolderEntry {
//     //     name: "New Folder".to_string(),
//     //     path: "/".to_string(),
//     // }).await?;
//     // client.move_picture(&path, "//New Folder").await?;
//
//     // let path = "/New Folder/Test2.unl".to_string();
//
//     // Delete
//     client.delete_picture(&path).await?;
//     // client.delete_folder("New Folder").await?;

    client.logout().await?;
    Ok(())
}
