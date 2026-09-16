use eveng::Client;
use eveng::labs::AddLabRequest;
use eveng::networks::AddNetworkRequest;
use eveng::nodes::AddNodeRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::login("http://localhost", "username", "password").await?;
    let root = client.folder("/")?;

    let lab = root.labs().add(AddLabRequest::new("Test")?).await?;

    let tmpl = client.system().node_template("vios").get().await?;

    let vios1 = lab
        .nodes()
        .add(
            AddNodeRequest::qemu(&tmpl)?
                .name("vios1")
                .position(100, 200)
                .ethernet(2),
        )
        .await?;

    let vios2 = lab
        .nodes()
        .add(
            AddNodeRequest::qemu(&tmpl)?
                .name("vios2")
                .position(200, 200)
                .ethernet(2),
        )
        .await?;

    vios1
        .ethernet(0)
        .connect_to_node(&vios2.ethernet(0))
        .await?;

    let cloud = lab.networks().add(AddNetworkRequest::new("pnet0")).await?;

    vios1.ethernet(1).connect_to_network(&cloud).await?;
    vios2.ethernet(1).connect_to_network(&cloud).await?;

    lab.nodes().start().await?;
    client.logout().await?;

    Ok(())
}
