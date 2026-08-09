/// result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// main error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("EVE-NG API error (status {status}): {message}")]
    Api {
        /// HTTP status code
        code: u16,
        /// API Status message (success, unauthorized, forbidden, fail, error)
        status: String,
        /// Error message from API
        message: String,
    },

    #[error("expected data in response but got none")]
    MissingData,

    #[error("Error while parsing an URL: {0:#?}")]
    Url(#[from] url::ParseError),
}
