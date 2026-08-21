mod common;

use eveng::Result;
use eveng::networks::{AddNetworkRequest, EditNetworkRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let networks = client.folder("/").lab("kimpfler-test.unl").networks();

    // List
    let resp = networks.list().await?;
    eprintln!("{:#?}", resp);

    // Add
    let network = networks.add(AddNetworkRequest::new("bridge")).await?;

    // Before
    let resp = network.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    network
        .edit(EditNetworkRequest::new().name("vmbr0"))
        .await?;

    // After
    let resp = network.get().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    network.delete().await?;

    client.logout().await?;
    Ok(())
}
