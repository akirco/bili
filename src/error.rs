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

    /// CSRF token not found
    #[error("CSRF token not found")]
    CsrfNotFound,

    /// Bilibili 风控/限流（HTTP 412 或 429）
    #[error(
        "Bilibili rate limited or risk-controlled (HTTP 412/429); retry later or supply a full cookie (including buvid3)"
    )]
    RateLimited,

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Cookie header build failed
    #[error("Cookie header build failed: {0}")]
    CookieBuild(String),

    /// API returned null data field
    #[error("API response data is null for {endpoint}")]
    NullData { endpoint: String },
}
