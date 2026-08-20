mod common;

use eveng::Result;
use eveng::nodes::{CreateNodeRequest, EditNodeRequest, NodeType, VpcsParams};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let lab = client.folder("/").lab("Test1");

    // List
    let resp = nodes.list().await?;
    eprintln!("{:#?}", resp);

    // Create
    let node = nodes
        .add(&CreateNodeRequest {
            template: "vpcs".to_string(),
            count: 1,
            name: "VPC".to_string(),
            icon: "PC-2D-Desktop-Generic-S.svg".to_string(),
            config: "0".to_string(),
            delay: 0,
            left: 0,
            top: 0,
            //postfix: 0,
            node_type: NodeType::Vpcs(VpcsParams { ethernet: 1 }),
        })
        .await?;

    // Before
    let resp = node.get().await?;
    eprintln!("{:#?}", resp);

    // Edit
    node.edit(&EditNodeRequest {
        config: None,
        delay: None,
        icon: None,
        left: None,
        name: Some("PC1".to_string()),
        top: None,
        node_type: None,
    })
    .await?;

    // After
    let resp = node.get().await?;
    eprintln!("{:#?}", resp);

    // List interfaces
    let resp = node.interfaces().await?;
    eprintln!("{:#?}", resp);

    // Delete
    node.delete().await?;

    client.logout().await?;
    Ok(())
}
