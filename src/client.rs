use crate::utils::number_from_string;
use crate::{Error, Result};
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub struct Client {
    pub url: String,
    pub username: String,
    pub password: String,
    pub lab_path: String,
    pub client: reqwest::Client,
}

#[derive(Debug, Serialize)]
pub struct LoginRequest {
    username: String,
    password: String,
    html5: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    code: u16,
    status: String,
    message: String,
    #[serde(default = "Option::default")]
    data: Option<T>,
}

impl<T> Response<T> {
    pub fn into_data(self) -> Result<T> {
        self.data.ok_or(Error::MissingData)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SystemStatus {
    cached: i32,
    cpu: i32,
    disk: i32,
    dynamips: i32,
    iol: i32,
    mem: i32,
    qemu: i32,
    qemu_version: String,
    swap: i32,
    version: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthInfo {
    email: String,
    lab: String,
    lang: String,
    name: String,
    role: String,
    #[serde(deserialize_with = "number_from_string")]
    tenant: i32,
    username: String,
}

impl Client {
    pub fn new(url: String, username: String, password: String, lab_path: String) -> Result<Self> {
        let client = reqwest::Client::builder().cookie_store(true).build()?;

        Ok(Self {
            url,
            username,
            password,
            lab_path,
            client,
        })
    }

    pub async fn login(&self) -> Result<()> {
        self.post::<(), LoginRequest>(
            "auth/login",
            &LoginRequest {
                username: self.username.clone(),
                password: self.password.clone(),
                // where to take this input?
                // should I take LoginRequest as a parameter?
                html5: 0,
            },
        )
        .await?;
        Ok(())
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
        B: Serialize + ?Sized,
    {
        let mut request = self
            .client
            .request(method, format!("{}/api/{}", self.url, endpoint));

        if let Some(body) = body {
            request = request.json(body);
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

        eprintln!("{}", text);
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

    pub async fn post<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        self.request(Method::POST, endpoint, Some(body)).await
    }

    pub async fn put<T, B>(&self, endpoint: &str, body: &B) -> Result<Response<T>>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
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
    use crate::{Client, Error};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[tokio::test]
    async fn test_bad_gateway_response() {
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

        let client = Client::new(
            server.uri(),
            "admin".to_string(),
            "eve".to_string(),
            "test.unl".to_string(),
        )
        .unwrap();

        let err = client.login().await.unwrap_err();

        match err {
            Error::Http { code, body } => {
                assert_eq!(code, 502);
                assert!(body.contains("502 Bad Gateway"));
            }
            other => panic!("expected HTTP error, got {other:?}"),
        }
    }
}
