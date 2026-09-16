use eveng::Client;
use eveng::labs::AddLabRequest;
use eveng::networks::AddNetworkRequest;
use eveng::nodes::AddNodeRequest;

use std::env;

async fn client_from_env() -> Result<Client, Box<dyn std::error::Error>> {
    let host = env::var("EVE_NG_URL").unwrap_or("http://127.0.0.1".to_string());
    let username = env::var("EVE_NG_USER").unwrap_or("admin".to_string());
    let password = env::var("EVE_NG_PASS").unwrap_or("eve".to_string());

    Ok(Client::login(host, username, password).await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = client_from_env().await?;

    let folder_path = env::var("EVE_NG_FOLDER").unwrap_or("/".to_string());
    let folder = client.folder(folder_path)?;

    let lab_name = env::var("EVE_NG_LAB").unwrap_or("Test".to_string());
    let lab = folder.labs().add(AddLabRequest::new(lab_name)?).await?;

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
                .position(100, 400)
                .ethernet(2),
        )
        .await?;

    vios1
        .ethernet(0)
        .connect_to_node(&vios2.ethernet(0))
        .await?;

    let cloud = lab.networks().add(
        AddNetworkRequest::new("pnet0")
            .name("Mgmt")
            .position(300, 300)
    ).await?;

    vios1.ethernet(1).connect_to_network(&cloud).await?;
    vios2.ethernet(1).connect_to_network(&cloud).await?;

    lab.nodes().start().await?;
    client.logout().await?;

    Ok(())
}
