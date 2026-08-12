use crate::folders::{Folder, Folders};
use crate::system::System;
use crate::users::{User, Users};
use crate::utils::number_from_string;
use crate::{Error, Result};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Main client for the EVE-NG API
#[derive(Debug, Clone)]
pub struct Client {
    base_url: Arc<Url>,
    api: reqwest::Client,
}

/// A builder to customize `Client` configuration
#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Arc<Url>,
    timeout: Duration,
    ssl_verify: bool,
    html5: u8,
}

impl ClientBuilder {
    fn new(base_url: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            base_url: Arc::new(Url::parse(base_url.as_ref())?),
            timeout: Duration::from_secs(30),
            ssl_verify: true,
            html5: 1,
        })
    }

    /// Set the request timeout. Defaults to 30 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set verification of SSL certificates. Defaults to true.
    pub fn ssl_verify(mut self, ssl_verify: bool) -> Self {
        self.ssl_verify = ssl_verify;
        self
    }

    /// Use the html5 console for EVE-NG. Defaults to 1.
    pub fn html5(mut self, html5: u8) -> Self {
        self.html5 = html5;
        self
    }

    /// Builds a `Client` and logs into the EVE-NG host.
    pub async fn login(
        &self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Client> {
        let client = Client {
            base_url: self.base_url.clone(),
            api: reqwest::Client::builder()
                .cookie_store(true)
                .timeout(self.timeout)
                .danger_accept_invalid_certs(!self.ssl_verify)
                .build()?,
        };

        #[derive(Serialize)]
        struct LoginRequest {
            username: String,
            password: String,
            html5: u8,
        }

        let params = &LoginRequest {
            username: username.into(),
            password: password.into(),
            html5: self.html5,
        };
        let _: Response<()> = client.post("auth/login", params).await?;

        Ok(client)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Response<T> {
    #[serde(deserialize_with = "number_from_string")]
    pub code: u16,
    pub status: String,
    pub message: String,
    #[serde(default = "Option::default")]
    pub data: Option<T>,
}

impl<T> Response<T> {
    pub(crate) fn into_data(self) -> Result<T> {
        self.data.ok_or(Error::MissingData)
    }
}

impl Client {
    /// Constructs a `Client` and logs into the EVE-NG instance
    pub async fn new(
        base_url: impl AsRef<str>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Client> {
        ClientBuilder::new(base_url)?
            .login(username, password)
            .await
    }

    /// Creates a `ClientBuilder` to customize a `Client`
    pub fn builder(base_url: impl AsRef<str>) -> Result<ClientBuilder> {
        ClientBuilder::new(base_url)
    }

    /// Logout of the EVE-NG instance
    pub async fn logout(&self) -> Result<()> {
        let _: Response<()> = self.get("auth/logout").await?;
        Ok(())
    }

    /// Access system-wide endpoints.
    pub fn system(&self) -> System {
        System::new(self.clone())
    }

    /// Access endpoints to manage folders on the host.
    pub fn folders(&self) -> Folders {
        Folders::new(self.clone())
    }

    /// Access endpoints to manage a specific folder.
    pub fn folder(&self, path: &str) -> Folder {
        Folder::new(self.clone(), path.trim_start_matches("/"))
    }

    /// Access endpoints to manage users on the host.
    pub fn users(&self) -> Users {
        Users::new(self.clone())
    }

    /// Access endpoints to manage a specific user.
    pub fn user(&self, username: &str) -> User {
        User::new(self.clone(), username)
    }

    async fn request<T, B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&B>,
    ) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self.base_url.join(&format!("api/{}", endpoint))?;
        let mut request = self.api.request(method, url);

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let text = response.text().await?;
            return Err(match serde_json::from_str::<Response<()>>(&text) {
                Ok(api) => Error::Api {
                    code: api.code,
                    status: api.status,
                    message: api.message,
                },
                Err(_) => Error::Api {
                    code: status.into(),
                    status: "error".to_string(),
                    message: text.chars().take(200).collect(),
                },
            });
        }

        Ok(response.json::<Response<T>>().await?)
    }

    /// Make a GET request to the API
    pub(crate) async fn get<T>(&self, endpoint: &str) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.request::<T, ()>(Method::GET, endpoint, None).await
    }

    /// Make a POST request to the API
    pub(crate) async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    /// Make a PUT request to the API
    pub(crate) async fn put<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::PUT, endpoint, Some(body)).await
    }

    /// Make a DELETE request to the API
    pub(crate) async fn delete<T>(&self, endpoint: &str) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.request::<T, ()>(Method::DELETE, endpoint, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Client, Error, Result};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn test_client_builder() -> Result<()> {
        let builder = Client::builder("http://eveng.example.com")?;

        assert_eq!(builder.base_url.as_str(), "http://eveng.example.com/");
        assert_eq!(builder.timeout, Duration::from_secs(30));
        assert_eq!(builder.html5, 1);
        assert!(builder.ssl_verify);

        Ok(())
    }

    #[test]
    fn test_client_builder_methods() -> Result<()> {
        let builder = Client::builder("http://eveng.example.com")?
            .timeout(Duration::from_secs(10))
            .ssl_verify(false)
            .html5(0);

        assert_eq!(builder.timeout, Duration::from_secs(10));
        assert_eq!(builder.html5, 0);
        assert!(!builder.ssl_verify);

        Ok(())
    }

    #[tokio::test]
    async fn test_api_error() -> Result<()> {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/auth/login"))
            .respond_with(ResponseTemplate::new(502).set_body_string(
                r#"<!DOCTYPE html>
<html>
<head>
    <title>502 Bad Gateway</title>
</head>
<body>
    <center>
        <h1>502 Bad Gateway</h1>
    </center>
    <hr>
    <center>nginx</center>
</body>
</html>"#,
            ))
            .mount(&server)
            .await;

        let client = Client::new(server.uri(), "admin", "eve").await;
        let err = client.unwrap_err();

        match err {
            Error::Api {
                code,
                status,
                message,
            } => {
                assert_eq!(code, 502);
                assert_eq!(status, "error".to_string());
                assert!(message.contains("502 Bad Gateway"));
            }
            other => panic!("expected HTTP error, got {other:?}"),
        }
        Ok(())
    }
}
