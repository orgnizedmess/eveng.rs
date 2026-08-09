use crate::utils::number_from_string;
use crate::{Error, Result};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Main client for the EVE-NG API
///
/// # Example
///
/// ```no_run
/// use eveng::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::builder("http://eveng.example.com", "Test.unl")?
///     .login("admin", "eve")
///     .await?;
/// client.system_status().await?;
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) base_url: Arc<Url>,
    pub(crate) api: reqwest::Client,
    pub(crate) lab_path: String,
}

#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Arc<Url>,
    lab_path: String,
    timeout: Duration,
    ssl_verify: bool,
    html5: u8,
}

impl ClientBuilder {
    pub fn new(base_url: impl AsRef<str>, lab_path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            base_url: Arc::new(Url::parse(base_url.as_ref())?),
            lab_path: lab_path.into(),
            timeout: Duration::from_secs(10),
            ssl_verify: true,
            html5: 1,
        })
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn ssl_verify(mut self, ssl_verify: bool) -> Self {
        self.ssl_verify = ssl_verify;
        self
    }

    pub fn html5(mut self, html5: u8) -> Self {
        self.html5 = html5;
        self
    }

    pub async fn login(
        &self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Client> {
        let client = Client {
            base_url: self.base_url.clone(),
            lab_path: self.lab_path.clone(),
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

        let _: Response<()> = client
            .post(
                "auth/login",
                &LoginRequest {
                    username: username.into(),
                    password: password.into(),
                    html5: self.html5,
                },
            )
            .await?;

        Ok(client)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    #[serde(deserialize_with = "number_from_string")]
    pub code: u16,
    pub status: String,
    pub message: String,
    #[serde(default = "Option::default")]
    pub data: Option<T>,
}

impl<T> Response<T> {
    pub fn into_data(self) -> Result<T> {
        self.data.ok_or(Error::MissingData)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    pub cached: i32,
    pub cpu: i32,
    pub disk: i32,
    pub dynamips: i32,
    pub iol: i32,
    pub mem: i32,
    pub qemu: i32,
    pub qemu_version: String,
    pub swap: i32,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthInfo {
    pub email: String,
    pub lab: String,
    pub lang: String,
    pub name: String,
    pub role: String,
    #[serde(deserialize_with = "number_from_string")]
    pub tenant: i32,
    pub username: String,
}

impl Client {
    pub fn builder(
        base_url: impl AsRef<str>,
        lab_path: impl Into<String>,
    ) -> Result<ClientBuilder> {
        Ok(ClientBuilder::new(base_url, lab_path)?)
    }

    pub async fn auth_info(&self) -> Result<AuthInfo> {
        self.get("auth").await?.into_data()
    }

    pub async fn logout(&self) -> Result<()> {
        self.get::<()>("auth/logout").await?;
        Ok(())
    }

    pub async fn system_status(&self) -> Result<SystemStatus> {
        self.get("status").await?.into_data()
    }

    pub async fn request<T, B>(
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

    pub async fn get<T>(&self, endpoint: &str) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.request::<T, ()>(Method::GET, endpoint, None).await
    }

    pub async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    pub async fn put<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::PUT, endpoint, Some(body)).await
    }

    pub async fn delete<T>(&self, endpoint: &str) -> Result<Response<T>>
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
        let builder = ClientBuilder::new("http://eveng.example.com", "Test.unl")?;

        assert_eq!(
            builder.base_url.as_str().trim_end_matches("/"),
            "http://eveng.example.com"
        );
        assert_eq!(builder.timeout, Duration::from_secs(10));
        assert_eq!(builder.html5, 1);
        assert!(builder.ssl_verify);

        Ok(())
    }

    #[test]
    fn test_builder_methods() -> Result<()> {
        let builder = ClientBuilder::new("http://eveng.example.com", "Test.unl")?
            .timeout(Duration::from_secs(30))
            .ssl_verify(false)
            .html5(0);

        assert_eq!(builder.timeout, Duration::from_secs(30));
        assert_eq!(builder.html5, 0);
        assert!(!builder.ssl_verify);

        Ok(())
    }

    #[tokio::test]
    async fn test_bad_gateway_response() -> Result<()> {
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

        let client = Client::builder(server.uri(), "Test.unl")?
            .login("admin", "eve")
            .await;
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
