mod common;

use eveng::Result;
use eveng::nodes::{AddNodeRequest, EditNodeRequest};

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;
    let lab = client.folder("/").lab("test");

    let veos = client.node_template("veos").get().await?;
    let req = AddNodeRequest::qemu(&veos)?;
    let node = lab.nodes().add(req).await?;

    // Edit
    let resp = node.get().await?;
    let req = EditNodeRequest::qemu(&resp)?.position(100, 200);
    node.edit(req).await?;

    // Delete
    node.delete().await?;

    client.logout().await?;
    Ok(())
}
