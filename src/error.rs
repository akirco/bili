use thiserror::Error;

/// Bilibili API client error type
///
/// Wraps HTTP, JSON parsing, API response, and other errors.
#[derive(Debug, Error)]
pub enum BiliError {
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Bilibili API returned error
    ///
    /// Contains error code and error message
    #[error("API error {code}: {message}")]
    Api { code: i64, message: String },

    /// Login required to perform this operation
    #[error("Login required")]
    LoginRequired,

    /// CSRF token not found
    #[error("CSRF token not found")]
    CsrfNotFound,

    /// WBI sign failed
    #[error("WBI sign failed")]
    WbiSignFailed,

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Cookie header build failed
    #[error("Cookie header build failed: {0}")]
    CookieBuild(String),
}