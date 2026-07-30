use eveng::Client;
use eveng::Result;

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

    eprintln!("{:#?}", client.topology().await?);
    eprintln!("{:#?}", client.links().await?);

    client.logout().await?;
    Ok(())
}
