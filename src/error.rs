use crate::client::Response;

/// result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// main error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Error while parsing an URL: {0:#?}")]
    Url(#[from] url::ParseError),

    #[error("EVE-NG API error (status {status}): {message}")]
    Api {
        /// HTTP status code
        code: u16,
        /// API Status message (success/unauthorized/forbidden/fail/error)
        status: String,
        /// Error message from API
        message: String,
    },

    #[error("Expected data in response but got none")]
    MissingData,

    #[error("Invalid name, contains invalid character '{0}'")]
    InvalidName(char),

    #[error("{0}")]
    Invalid(String),
}

impl Error {
    pub fn from_response(code: reqwest::StatusCode, body: String) -> Self {
        match serde_json::from_str::<Response<()>>(&body) {
            Ok(api) => Error::Api {
                code: api.code,
                status: api.status,
                message: api.message,
            },
            Err(_) => Error::Api {
                code: code.into(),
                status: "error".to_string(),
                message: body.chars().take(200).collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_api_error() {
        let body = r#"<html><head><title>Slim Application Error</title><style>body{margin:0;padding:30px;font:12px/1.5 Helvetica,Arial,Verdana,sans-serif;}h1{margin:0;font-size:48px;font-weight:normal;line-height:48px;}strong{display:inline-block;width:65px;}</style></head><body><h1>Slim Application Error</h1><p>The application could not run because of the following error:</p><h2>Details</h2><div><strong>Type:</strong> ErrorException</div><div><strong>Code:</strong> 2</div><div><strong>Message:</strong> Undefined array key \"rows\"</div><div><strong>File:</strong> /opt/unetlab/html/includes/api_authentication.php</div><div><strong>Line:</strong> 193</div><h2>Trace</h2><pre><div>#0 /opt/unetlab/html/includes/api_authentication.php(193): Slim\\Slim::handleErrors()</div><div>#1 /opt/unetlab/html/api.php(129): apiLogin()</div><div>#2 [internal function]: {closure}()</div><div>#3 /opt/unetlab/html/includes/Slim/Route.php(468): call_user_func_array()</div><div>#4 /opt/unetlab/html/includes/Slim/Slim.php(1357): Slim\\Route->dispatch()</div><div>#5 /opt/unetlab/html/includes/Slim/Middleware/Flash.php(85): Slim\\Slim->call()</div><div>#6 /opt/unetlab/html/includes/Slim/Middleware/MethodOverride.php(92): Slim\\Middleware\\Flash->call()</div><div>#7 /opt/unetlab/html/includes/Slim/Middleware/PrettyExceptions.php(67): Slim\\Middleware\\MethodOverride->call()</div><div>#8 /opt/unetlab/html/includes/Slim/Slim.php(1302): Slim\\Middleware\\PrettyExceptions->call()</div><div>#9 /opt/unetlab/html/api.php(1368): Slim\\Slim->run()</div><div>#10 {main}</pre></body></html>"#;

        let err =
            Error::from_response(reqwest::StatusCode::INTERNAL_SERVER_ERROR, body.to_string());
        assert!(matches!(err, Error::Api { code: 500, .. }));
    }

    #[test]
    fn json_api_error() {
        let body = r#"{"code": 404, "message": "Requested folder does not exist (60008).", "status": "fail"}"#;

        let err = Error::from_response(reqwest::StatusCode::NOT_FOUND, body.to_string());
        assert!(matches!(err, Error::Api { code: 404, .. }));
    }
}
