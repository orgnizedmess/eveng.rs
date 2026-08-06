use crate::utils::number_from_string;
use crate::{Error, Result};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Client for the EVE-NG API
///
/// # Example
///
/// ```no_run
/// use eveng::Client;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Client::new("http://localhost", "Test.unl")?
///     .login("admin", "eve")
///     .await?;
/// client.nodes().await?;
/// #   Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) base_url: Url,
    pub(crate) client: reqwest::Client,
    timeout: Duration,
    pub(crate) lab_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    #[serde(deserialize_with = "number_from_string")]
    pub code: i32,
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
    pub fn new(base_url: impl AsRef<str>, lab_path: impl Into<String>) -> Result<Self> {
        Ok(Self {
            base_url: Url::parse(base_url.as_ref())?,
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(10),
            lab_path: lab_path.into(),
        })
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn login(
        &mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Self> {
        self.client = reqwest::Client::builder()
            .timeout(self.timeout)
            .cookie_store(true)
            .build()?;

        #[derive(Serialize)]
        struct LoginRequest {
            username: String,
            password: String,
            html5: i32,
        }

        self.post::<(), LoginRequest>(
            "/auth/login",
            LoginRequest {
                username: username.into(),
                password: password.into(),
                html5: 1,
            },
        )
        .await?;
        Ok(self.clone())
    }

    pub async fn auth_info(&self) -> Result<AuthInfo> {
        self.get("/auth").await?.into_data()
    }

    pub async fn logout(&self) -> Result<()> {
        self.get::<()>("/auth/logout").await?;
        Ok(())
    }

    pub async fn system_status(&self) -> Result<SystemStatus> {
        self.get("/status").await?.into_data()
    }

    pub async fn request<T, B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<B>,
    ) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let uri = self.base_url.join(&format!("/api{}", endpoint))?;
        let mut request = self.client.request(method, uri);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            return Err(Error::Http {
                code: status,
                body: text,
            });
        }

        let response: Response<T> = serde_json::from_str(&text)?;

        match response.code {
            200..=299 => Ok(response),
            _ => Err(Error::Api {
                code: response.code,
                status: response.status.to_string(),
                message: response.message.to_string(),
                body: text,
            }),
        }
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<Response<T>>
    where
        T: DeserializeOwned,
    {
        self.request::<T, ()>(Method::GET, endpoint, None).await
    }

    pub async fn post<T, B>(&self, endpoint: &str, body: B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::POST, endpoint, Some(&body)).await
    }

    pub async fn put<T, B>(&self, endpoint: &str, body: B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::PUT, endpoint, Some(&body)).await
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
    use crate::{Client, Error, Result};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

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

        let client = Client::new(server.uri(), "test.unl")?
            .login("admin", "eve")
            .await;
        let err = client.unwrap_err();

        match err {
            Error::Http { code, body } => {
                assert_eq!(code, 502);
                assert!(body.contains("502 Bad Gateway"));
            }
            other => panic!("expected HTTP error, got {other:?}"),
        }
        Ok(())
    }
}
