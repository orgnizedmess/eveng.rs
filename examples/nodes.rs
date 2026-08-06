mod common;

use eveng::Result;
use eveng::nodes::{
    CreateNodeRequest, CreateNodeResponse, EditNodeRequest, NodeType, Nodes, VpcsParams,
};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;

    // List
    let resp: Nodes = client.nodes().await?;
    eprintln!("{:#?}", resp);

    // Create
    let resp: CreateNodeResponse = client
        .add_node(CreateNodeRequest {
            template: "vpcs".to_string(),
            count: 1,
            name: "VPC".to_string(),
            icon: "PC-2D-Desktop-Generic-S.svg".to_string(),
            config: "0".to_string(),
            delay: 0,
            left: 0,
            top: 0,
            postfix: 0,
            node_type: NodeType::Vpcs(VpcsParams { ethernet: 1 }),
        })
        .await?;

    let id = resp.id;

    // Before
    client.node(id).await?;

    // Edit
    client
        .edit_node(
            id,
            EditNodeRequest {
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
