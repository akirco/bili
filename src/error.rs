use std::fmt;

/// Bilibili API client error type
///
/// Wraps HTTP, JSON parsing, API response, and other errors.
#[derive(Debug)]
pub enum BiliError {
    /// HTTP request error
    Http(reqwest::Error),
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// Bilibili API returned error
    ///
    /// Contains error code and error message
    Api { code: i64, message: String },
    /// Login required to perform this operation
    LoginRequired,
    /// CSRF token not found
    CsrfNotFound,
    /// WBI sign failed
    WbiSignFailed,
    /// Parse error
    Parse(String),
}

impl fmt::Display for BiliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BiliError::Http(e) => write!(f, "HTTP error: {}", e),
            BiliError::Json(e) => write!(f, "JSON error: {}", e),
            BiliError::Api { code, message } => write!(f, "API error {}: {}", code, message),
            BiliError::LoginRequired => write!(f, "Login required"),
            BiliError::CsrfNotFound => write!(f, "CSRF token not found"),
            BiliError::WbiSignFailed => write!(f, "WBI sign failed"),
            BiliError::Parse(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl std::error::Error for BiliError {}

impl From<reqwest::Error> for BiliError {
    fn from(e: reqwest::Error) -> Self {
        BiliError::Http(e)
    }
}

impl From<serde_json::Error> for BiliError {
    fn from(e: serde_json::Error) -> Self {
        BiliError::Json(e)
    }
}
