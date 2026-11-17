//! Rendering a command result to stdout and deciding the process exit code.
//!
//! Both concerns belong to the CLI process, not the library: a caller of
//! `bili::video` gets typed results and errors, and decides for itself how to
//! present them.

use serde_json::Value;

/// How much of the JSON result to show.
#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
    /// Pretty-printed with the envelope (`code` / `message` / `data`).
    Pretty,
    /// Single-line, unmodified.
    Raw,
    /// Pretty-printed `data` field only.
    Compact,
}

/// Print `value` to stdout according to `mode`.
pub fn print_value(value: &Value, mode: OutputMode) {
    let rendered = match mode {
        OutputMode::Raw => serde_json::to_string(value),
        OutputMode::Compact => {
            let output = value.get("data").unwrap_or(value);
            serde_json::to_string_pretty(output)
        }
        OutputMode::Pretty => serde_json::to_string_pretty(value),
    };
    // Serialising a `Value` cannot fail.
    println!("{}", rendered.expect("serialising a Value cannot fail"));
}

/// An API-level failure: the HTTP call succeeded but `code != 0`.
#[derive(Debug, thiserror::Error)]
#[error("API error {code}: {message}")]
pub struct ApiError {
    /// The `code` field returned by the endpoint.
    pub code: i64,
    /// The `message` field, or `unknown error`.
    pub message: String,
}

/// Convert a non-zero `code` in the response envelope into an error.
///
/// Scripts branch on the exit status, so a soft API error has to fail the
/// process rather than print `code: -400` and exit 0.
///
/// # Errors
///
/// Returns [`ApiError`] when `value.code` exists and is non-zero.
pub fn check_api_error(value: &Value) -> Result<(), ApiError> {
    let Some(code) = value.get("code").and_then(Value::as_i64) else {
        return Ok(());
    };
    if code == 0 {
        return Ok(());
    }
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("unknown error")
        .to_string();
    Err(ApiError { code, message })
}
