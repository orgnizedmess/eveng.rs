use eveng::{Client, Result};

pub async fn test_client() -> Result<Client> {
    Client::builder("http://192.168.0.141", "Test.unl")?
        .login("admin", "eve")
        .await
}
