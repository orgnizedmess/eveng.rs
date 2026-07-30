use crate::{Error, Result};
use crate::utils::number_from_string;
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
    #[serde(deserialize_with = "number_from_string")]
    code: i32,
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

        let response: Response<T> = request.send().await?.json().await?;

        match response.code {
            200..=299 => Ok(response),
            _ => Err(Error::Api {
                code: response.code,
                status: response.status.to_string(),
                message: response.message.to_string(),
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
