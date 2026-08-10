mod common;

use eveng::Result;
use eveng::networks::{CreateNetworkRequest, EditNetworkRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let networks = client.folder("/").lab("Test.unl").networks();

    // List
    let resp = networks.list().await?;
    eprintln!("{:#?}", resp);

    // Add
    let network = networks
        .add(&CreateNetworkRequest {
            count: 1,
            visibility: 1,
            name: None,
            network_type: "bridge".to_string(),
            icon: None,
            left: None,
            postfix: None,
            top: None,
        })
        .await?;

    // Before
    let resp = network.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    network
        .edit(&EditNetworkRequest {
            name: Some("vmbr0".to_string()),
            network_type: None,
            icon: None,
            left: None,
            top: None,
            visibility: None,
            postfix: None,
        })
        .await?;

    // After
    let resp = network.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    network.delete().await?;

    client.logout().await?;
    Ok(())
}
