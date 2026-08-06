mod common;

use eveng::Result;

#[tokio::main]
pub async fn main() -> Result<()> {
    let client = common::test_client().await?;

    eprintln!("{:#?}", client.topology().await?);
    eprintln!("{:#?}", client.links().await?);

    client.logout().await?;
    Ok(())
}
