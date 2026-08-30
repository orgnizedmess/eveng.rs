mod common;

use eveng::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let lab = client.folder("/").lab("test");
    eprintln!("{:#?}", lab.links().await?);

    let node1 = lab.node(7);
    eprintln!("{:#?}", node1.interfaces().list().await?);
    let node2 = lab.node(8);

    // node -> node (ethernet)
    let if1 = node1.ethernet(0);
    let if2 = node2.ethernet(0);
    if1.connect_to_node(&if2).await?;

    if1.disconnect().await?;

    // node -> network
    let if1 = node1.ethernet(0);
    let cloud = lab.network(1);
    if1.connect_to_network(&cloud).await?;

    if1.disconnect().await?;

    // node -> node (serial)
    let if1 = node1.serial(1);
    let if2 = node2.serial(1);
    if1.connect_to_node(&if2).await?;

    if1.disconnect().await?;

    client.logout().await?;
    Ok(())
}
