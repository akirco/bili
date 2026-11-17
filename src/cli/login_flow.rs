//! QR-code login: poll the status endpoint until the user confirms.
//!
//! Both the terminal rendering of the QR code and the progress notices are
//! injected as callbacks, so this module owns no presentation: the CLI passes
//! a dense-unicode renderer and a stderr printer (see `cli::mod`).

use crate::BiliClient;
use std::time::Duration;

/// How long to wait between status polls.
pub const POLL_INTERVAL: Duration = Duration::from_secs(2);
/// Give up after this long.
pub const TIMEOUT: Duration = Duration::from_secs(180);
/// Tolerate this many consecutive transport errors before giving up.
pub const MAX_CONSECUTIVE_ERRORS: u32 = 3;

/// Why a QR login did not complete.
#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    /// Generating the QR code failed.
    #[error("failed to generate QR code: {0}")]
    QrCodeFailed(String),
    /// The QR response carried no `data` payload.
    #[error("no data in QR code response")]
    MissingQrData,
    /// The caller's `render_qr` callback failed.
    #[error("QR code render failed: {0}")]
    Render(String),
    /// The status response carried no `data` payload.
    #[error("no data in status response")]
    MissingStatusData,
    /// The user did not confirm in time.
    #[error("login timed out; run `bili login` again")]
    Timeout,
    /// Too many consecutive transport errors while polling.
    #[error("login polling failed {count} times in a row: {source}")]
    PollFailed {
        /// Consecutive failure count at the point of giving up.
        count: u32,
        /// The last transport error.
        source: crate::BiliError,
    },
    /// The server rejected the login.
    #[error("login failed: {0}")]
    Rejected(String),
    /// Login succeeded but the cookie URL was absent.
    #[error("login succeeded but the response contains no cookie URL; please retry")]
    MissingCookieUrl,
    /// Cookies arrived but `SESSDATA` was not among them.
    #[error("login returned cookies but SESSDATA is missing; please retry")]
    MissingSession,
    /// The QR code expired before confirmation.
    #[error("QR code expired, please try again")]
    Expired,
    /// A status code this client does not know.
    #[error("unexpected login status: {0}")]
    UnexpectedStatus(i64),
    /// A request failed outside the polling loop.
    #[error(transparent)]
    Bili(#[from] crate::BiliError),
}

/// Run the QR login flow, handing the QR payload URL to `render_qr` and
/// progress messages to `notify`.
///
/// Both presentation hooks are injected so this module writes nothing to
/// stdout/stderr itself: the CLI decides how a QR code is drawn and where
/// progress notices go (today: a dense-unicode block, and stderr).
///
/// On success the cookies are written to the client's cookie store.
///
/// # Errors
///
/// Returns [`LoginError`] on timeout, expiry, rejection, missing cookies, or
/// repeated transport failures.
pub async fn qr_login(
    client: &BiliClient,
    mut render_qr: impl FnMut(&str) -> Result<(), LoginError>,
    mut notify: impl FnMut(&str),
) -> Result<(), LoginError> {
    let resp = client.login().qrcode().await?;
    if resp.code != 0 {
        return Err(LoginError::QrCodeFailed(resp.message));
    }
    let data = resp.data.ok_or(LoginError::MissingQrData)?;

    render_qr(&data.url)?;

    let deadline = tokio::time::Instant::now() + TIMEOUT;
    let mut consecutive_errors = 0u32;
    let mut announced_scan = false;

    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err(LoginError::Timeout);
        }
        tokio::time::sleep(POLL_INTERVAL).await;

        let status = match client.login().qrcode_status(&data.qrcode_key).await {
            Ok(status) => {
                consecutive_errors = 0;
                status
            }
            Err(source) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    return Err(LoginError::PollFailed {
                        count: consecutive_errors,
                        source,
                    });
                }
                notify(&format!(
                    "Login poll failed ({consecutive_errors}/{MAX_CONSECUTIVE_ERRORS}): {source}; retrying..."
                ));
                continue;
            }
        };

        if status.code != 0 {
            return Err(LoginError::Rejected(status.message));
        }
        let s = status.data.ok_or(LoginError::MissingStatusData)?;

        match s.code {
            0 => {
                let Some(url) = s.url else {
                    return Err(LoginError::MissingCookieUrl);
                };
                client.parse_cookies(&url)?;
                if !client.is_logged_in() {
                    return Err(LoginError::MissingSession);
                }
                client.flush_cookies();
                return Ok(());
            }
            86090 => {
                if !announced_scan {
                    notify("QR code scanned, waiting for confirmation...");
                    announced_scan = true;
                }
            }
            86101 => {}
            86038 => {
                return Err(LoginError::Expired);
            }
            other => {
                return Err(LoginError::UnexpectedStatus(other));
            }
        }
    }
}
