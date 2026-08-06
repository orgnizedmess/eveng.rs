/// result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// main error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("HTTP error: {code} {body}")]
    Http {
        /// http status code
        code: reqwest::StatusCode,
        /// response body
        body: String,
    },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// api returned an error response
    #[error("EVE-NG API error (status {status}): {message}")]
    Api {
        /// http status code
        code: i32,
        /// http status message
        status: String,
        /// error message from api
        message: String,
        /// response body
        body: String,
    },

    #[error("expected data in response but got none")]
    MissingData,

    #[error(transparent)]
    Url(#[from] url::ParseError),
}
