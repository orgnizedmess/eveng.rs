use eveng::Client;
use eveng::Result;
use eveng::nodes::{
    CreateNodeRequest, CreateNodeResponse, EditNodeRequest, Nodes, NodeType
};

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

    // List
    let resp: Nodes = client.nodes().await?;
    eprintln!("{:#?}", resp);

    // Create
    let resp: CreateNodeResponse = client
        .add_node(&CreateNodeRequest {
            template: "vpcs".to_string(),
            count: 1,
            name: "VPC".to_string(),
            icon: "PC-2D-Desktop-Generic-S.svg".to_string(),
            config: "0".to_string(),
            delay: 0,
            left: 0,
            top: 0,
            postfix: 0,
            node_type: NodeType::Vpcs,
        })
        .await?;

    let id = resp.id;

    // Before
    client.node(id).await?;

    // Edit
    client
        .edit_node(
            id,
            &EditNodeRequest {
                config: None,
                delay: None,
                icon: None,
                left: None,
                name: Some("PC1".to_string()),
                top: None,
                node_type: None,
            },
        )
        .await?;

    // After
    client.node(id).await?;

    // List interfaces
    let resp = client.interfaces(id).await?;
    eprintln!("{:#?}", resp);

    // Delete
    client.delete_node(id).await?;

    client.logout().await?;
    Ok(())
}
