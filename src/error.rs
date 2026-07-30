/// result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// main error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// api returned an error response
    #[error("EVE-NG API error (status {status}): {message}")]
    Api {
        /// http status code
        code: i32,
        /// http status message
        status: String,
        /// error message from api
        message: String,
    },

    #[error("expected data in response but got none")]
    MissingData,
}
