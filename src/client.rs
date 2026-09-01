use crate::folders::{FolderClient, FoldersClient};
use crate::system::SystemClient;
use crate::users::{UserClient, UserName, UsersClient};
use crate::utils::number_from_string;
use crate::{Error, Result};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Main entry point: A client for the EVE-NG API.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) base_url: Arc<Url>,
    api: reqwest::Client,
}

/// A builder to customize [`Client`] configuration.
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

    /// Builds a [`Client`] and logs into the EVE-NG instance.
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

        let username = UserName::new(username.into())?;

        #[derive(Serialize)]
        struct LoginRequest {
            username: String,
            password: String,
            html5: u8,
        }

        let params = &LoginRequest {
            username: username.to_string(),
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
    /// Creates a new API client.
    pub async fn new(
        base_url: impl AsRef<str>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Client> {
        ClientBuilder::new(base_url)?
            .login(username, password)
            .await
    }

    /// Returns a builder to customize client configuration.
    pub fn builder(base_url: impl AsRef<str>) -> Result<ClientBuilder> {
        ClientBuilder::new(base_url)
    }

    /// Logs out of the EVE-NG instance.
    pub async fn logout(&self) -> Result<()> {
        let _: Response<()> = self.get("auth/logout").await?;
        Ok(())
    }

    /// Returns a client for system-level information.
    pub fn system(&self) -> SystemClient {
        SystemClient::new(self.clone())
    }

    /// Returns a client to manage folders.
    pub fn folders(&self) -> FoldersClient {
        FoldersClient::new(self.clone())
    }

    /// Returns a client to manage a single folder.
    pub fn folder(&self, path: &str) -> Result<FolderClient> {
        FolderClient::new(self.clone(), path)
    }

    /// Returns a client to manage users.
    pub fn users(&self) -> UsersClient {
        UsersClient::new(self.clone())
    }

    /// Returns a client to manage a single user.
    pub fn user(&self, username: impl Into<String>) -> Result<UserClient> {
        UserClient::new(self.clone(), username)
    }

    pub fn node_template(&self, name: impl Into<String>) -> TemplateClient {
        TemplateClient::new(self.clone(), name)
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
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::from_response(status, text));
        }

        Ok(serde_json::from_str::<Response<T>>(&text).unwrap())
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

    #[test]
    fn valid_client_builder() {
        let builder = Client::builder("http://eveng.example.com").unwrap();
        assert_eq!(builder.base_url.as_str(), "http://eveng.example.com/");
        assert_eq!(builder.timeout, Duration::from_secs(30));
        assert_eq!(builder.html5, 1);
        assert!(builder.ssl_verify);
    }

    #[test]
    fn invalid_client_builder() {
        let err = Client::builder("eveng.example.com").unwrap_err();
        assert!(matches!(err, Error::InvalidUrl(_)));
    }

    #[test]
    fn client_builder_methods() -> Result<()> {
        let builder = Client::builder("http://eveng.example.com")?
            .timeout(Duration::from_secs(10))
            .ssl_verify(false)
            .html5(0);

        assert_eq!(builder.timeout, Duration::from_secs(10));
        assert_eq!(builder.html5, 0);
        assert!(!builder.ssl_verify);
        Ok(())
    }
}
