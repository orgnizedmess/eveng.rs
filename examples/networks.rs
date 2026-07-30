use eveng::Client;
use eveng::Result;
use eveng::networks::Network;

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

    let resp = client.network_types().await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Add
    let resp: eveng::networks::CreateNetworkResponse = client
        .add_network(&eveng::networks::CreateNetworkRequest {
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

    let id = resp.id;

    // Before
    let resp: Network = client.network(id).await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Edit
    client
        .edit_network(
            id,
            &eveng::networks::EditNetworkRequest {
                name: Some("vmbr0".to_string()),
                network_type: None,
                icon: None,
                left: None,
                top: None,
                visibility: None,
                postfix: None,
            },
        )
        .await?;

    // After
    let resp: Network = client.network(id).await?;
    eprintln!("{}", serde_json::to_string_pretty(&resp).unwrap());

    // Delete
    client.delete_network(id).await?;

    client.logout().await?;
    Ok(())
}
