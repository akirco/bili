//! Core HTTP client: shared plumbing, request transport, and identity.
//!
//! [`BiliClient`] is the single entry point every domain client is built on.
//! Its implementation is split by concern:
//!
//! - [`params`] — query/form construction ([`Params`], [`FormBuilder`]);
//! - [`transport`] — WBI signing, retries, and the `get` / `post` entry points;
//! - [`auth`] — cookie parsing/persistence, `csrf`, `is_logged_in`, buvid.
//!
//! What stays here is the type definitions, construction ([`BiliClient::new`] /
//! [`BiliClient::builder`]) and the per-domain accessors (`client.video()`, …).

mod auth;
mod params;
mod transport;

pub use params::{FormBuilder, FormData, Params};

use crate::cookie::{SharedCookieStore, default_cookie_path, new_shared};
use crate::error::BiliError;
use crate::wbi::WbiCache;
use reqwest::ClientBuilder as ReqwestClientBuilder;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

const BUVID_SPI_URL: &str = "https://api.bilibili.com/x/frontend/finger/spi";

fn is_bilibili_host(url: &reqwest::Url) -> bool {
    url.host_str().is_some_and(|host| {
        let host = host.to_ascii_lowercase();
        host == "bilibili.com"
            || host.ends_with(".bilibili.com")
            || host == "biligame.com"
            || host.ends_with(".biligame.com")
    })
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BiliResponse<T> {
    pub code: i64,
    #[serde(default)]
    pub message: String,
    pub ttl: Option<i64>,
    pub data: Option<T>,
}

impl<T> BiliResponse<T> {
    /// Return `Ok(())` when the API reported success (`code == 0`),
    /// otherwise an [`BiliError::Api`].
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::Api`] if `code` is non-zero.
    pub fn ensure_ok(&self) -> Result<(), BiliError> {
        if self.code == 0 {
            Ok(())
        } else {
            Err(BiliError::Api {
                code: self.code,
                message: self.message.clone(),
            })
        }
    }

    /// Consume the response, returning the `data` field.
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::Api`] if `code` is non-zero, or
    /// [`BiliError::NullData`] if `data` is `None`.
    pub fn into_data(self, endpoint: &str) -> Result<T, BiliError> {
        if self.code != 0 {
            return Err(BiliError::Api {
                code: self.code,
                message: self.message,
            });
        }
        self.data.ok_or_else(|| BiliError::NullData {
            endpoint: endpoint.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct BiliClient {
    pub(crate) inner: Arc<BiliClientInner>,
}

/// Shared plumbing behind [`BiliClient`] and the per-feature API clients.
///
/// Crate-internal: users go through [`BiliClient`] and the `xxx()` accessors,
/// never through this type.
#[derive(Clone)]
pub(crate) struct BiliClientInner {
    pub(crate) http: reqwest::Client,
    /// Client without a total-request timeout, used for large downloads.
    pub(crate) download_http: reqwest::Client,
    pub(crate) cookie_store: SharedCookieStore,
    pub(crate) wbi_cache: Arc<RwLock<Option<WbiCache>>>,
    pub(crate) wbi_ttl: Duration,
}

pub struct BiliClientBuilder {
    timeout: Duration,
    connect_timeout: Duration,
    headers: HeaderMap,
    user_agent: &'static str,
    referer: &'static str,
    wbi_ttl: Duration,
    cookie_path: PathBuf,
}

impl Default for BiliClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            headers: HeaderMap::new(),
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            referer: "https://www.bilibili.com",
            wbi_ttl: Duration::from_hours(6),
            cookie_path: default_cookie_path(),
        }
    }
}

impl BiliClientBuilder {
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    #[must_use]
    pub fn user_agent(mut self, user_agent: &'static str) -> Self {
        self.user_agent = user_agent;
        self
    }

    #[must_use]
    pub fn referer(mut self, referer: &'static str) -> Self {
        self.referer = referer;
        self
    }

    #[must_use]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: reqwest::header::IntoHeaderName,
        V: Into<HeaderValue>,
    {
        self.headers.insert(key, value.into());
        self
    }

    #[must_use]
    pub fn wbi_ttl(mut self, ttl: Duration) -> Self {
        self.wbi_ttl = ttl;
        self
    }

    /// Cookie 持久化文件路径（默认 `~/.config/bili/cookies.json`）
    #[must_use]
    pub fn cookie_path(mut self, path: PathBuf) -> Self {
        self.cookie_path = path;
        self
    }

    pub fn build(self) -> Result<BiliClient, BiliError> {
        let mut headers = self.headers;
        headers.insert(USER_AGENT, HeaderValue::from_static(self.user_agent));
        headers.insert("Referer", HeaderValue::from_static(self.referer));

        let http = ReqwestClientBuilder::new()
            .default_headers(headers.clone())
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 10 {
                    return attempt.error("too many redirects");
                }
                let Some(previous) = attempt.previous().last() else {
                    return attempt.follow();
                };
                let next = attempt.url();
                let same_origin = previous.scheme() == next.scheme()
                    && previous.host_str() == next.host_str()
                    && previous.port_or_known_default() == next.port_or_known_default();
                // Allow redirects inside Bilibili domains (cookies are scoped
                // to .bilibili.com); stop cross-site redirects so the Cookie
                // header is never forwarded to an unrelated host.
                let both_bilibili = is_bilibili_host(previous) && is_bilibili_host(next);
                if same_origin || both_bilibili {
                    attempt.follow()
                } else {
                    attempt.stop()
                }
            }))
            .build()?;

        // `ClientBuilder::timeout` covers the whole response body, which would
        // abort a large video download after 30 seconds. Downloads get their
        // own client with only a read timeout.
        let download_http = ReqwestClientBuilder::new()
            .default_headers(headers)
            .connect_timeout(self.connect_timeout)
            .read_timeout(Duration::from_secs(60))
            .build()?;

        Ok(BiliClient {
            inner: Arc::new(BiliClientInner {
                http,
                download_http,
                cookie_store: new_shared(self.cookie_path),
                wbi_cache: Arc::new(RwLock::new(None)),
                wbi_ttl: self.wbi_ttl,
            }),
        })
    }
}

impl BiliClient {
    pub fn new() -> Result<Self, BiliError> {
        BiliClientBuilder::default().build()
    }

    #[must_use]
    pub fn builder() -> BiliClientBuilder {
        BiliClientBuilder::default()
    }

    #[cfg(feature = "video")]
    #[must_use]
    pub fn video(&self) -> crate::video::VideoClient {
        crate::video::VideoClient::new(self.inner.clone())
    }

    #[cfg(feature = "user")]
    #[must_use]
    pub fn user(&self) -> crate::user::UserClient {
        crate::user::UserClient::new(self.inner.clone())
    }

    #[cfg(feature = "search")]
    #[must_use]
    pub fn search(&self) -> crate::search::SearchClient {
        crate::search::SearchClient::new(self.inner.clone())
    }

    #[cfg(feature = "comment")]
    #[must_use]
    pub fn comment(&self) -> crate::comment::CommentClient {
        crate::comment::CommentClient::new(self.inner.clone())
    }

    #[cfg(feature = "fav")]
    #[must_use]
    pub fn fav(&self) -> crate::fav::FavClient {
        crate::fav::FavClient::new(self.inner.clone())
    }

    #[cfg(feature = "danmaku")]
    #[must_use]
    pub fn danmaku(&self) -> crate::danmaku::DanmakuClient {
        crate::danmaku::DanmakuClient::new(self.inner.clone())
    }

    #[cfg(feature = "audio")]
    #[must_use]
    pub fn audio(&self) -> crate::audio::AudioClient {
        crate::audio::AudioClient::new(self.inner.clone())
    }

    #[cfg(feature = "history")]
    #[must_use]
    pub fn history(&self) -> crate::history::HistoryClient {
        crate::history::HistoryClient::new(self.inner.clone())
    }

    #[cfg(feature = "login")]
    #[must_use]
    pub fn login(&self) -> crate::login::LoginClient {
        crate::login::LoginClient::new(self.inner.clone())
    }

    #[cfg(feature = "subtitle")]
    #[must_use]
    pub fn subtitle(&self) -> crate::subtitle::SubtitleClient {
        crate::subtitle::SubtitleClient::new(self.inner.clone())
    }

    #[cfg(feature = "action")]
    #[must_use]
    pub fn action(&self) -> crate::action::ActionClient {
        crate::action::ActionClient::new(self.inner.clone())
    }

    #[cfg(feature = "live")]
    #[must_use]
    pub fn live(&self) -> crate::live::LiveClient {
        crate::live::LiveClient::new(self.inner.clone())
    }
}

/// Detect a WBI signature failure.
///
/// Bilibili may return `code: -352/-403`, or HTTP 200 with a `v_voucher` in
/// `data`; both mean the cached `mixin_key` is stale.
fn is_wbi_failure(value: &serde_json::Value) -> bool {
    let code = value
        .get("code")
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0);
    if matches!(code, -352 | -403) {
        return true;
    }
    value
        .get("data")
        .and_then(|data| data.get("v_voucher"))
        .is_some_and(|voucher| !voucher.is_null())
}

/// Extract `data.b_3` / `data.b_4` from the fingerprint endpoint.
fn parse_buvid(value: &serde_json::Value) -> Option<(String, Option<String>)> {
    let buvid3 = value.get("data")?.get("b_3")?.as_str()?.to_string();
    if buvid3.is_empty() {
        return None;
    }
    let buvid4 = value
        .get("data")
        .and_then(|d| d.get("b_4"))
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(String::from);
    Some((buvid3, buvid4))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_buvid_extracts_both_values() {
        let value = serde_json::json!({"code": 0, "data": {"b_3": "b3-value", "b_4": "b4-value"}});
        assert_eq!(
            parse_buvid(&value),
            Some(("b3-value".to_string(), Some("b4-value".to_string())))
        );
    }

    #[test]
    fn parse_buvid_tolerates_missing_or_empty_values() {
        assert_eq!(
            parse_buvid(&serde_json::json!({"data": {"b_4": "b4"}})),
            None
        );
        assert_eq!(
            parse_buvid(&serde_json::json!({"data": {"b_3": "", "b_4": "b4"}})),
            None
        );
        assert_eq!(
            parse_buvid(&serde_json::json!({"data": {"b_3": "b3"}})),
            Some(("b3".to_string(), None))
        );
    }

    #[test]
    fn client_builder_defaults() {
        let b = BiliClientBuilder::default();
        assert_eq!(b.timeout, Duration::from_secs(30));
        assert_eq!(b.connect_timeout, Duration::from_secs(10));
        assert_eq!(b.wbi_ttl, Duration::from_hours(6));
    }

    #[test]
    fn client_builder_custom() {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Never point tests at the user's real cookie jar.
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let cookie_path = std::env::temp_dir().join(format!("bili_client_test_{ts}.json"));

        let client = BiliClientBuilder::default()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .wbi_ttl(Duration::from_mins(30))
            .cookie_path(cookie_path.clone())
            .build()
            .unwrap();
        assert_eq!(client.inner.wbi_ttl, Duration::from_mins(30));
        drop(client);
        let _ = std::fs::remove_file(cookie_path);
    }

    #[test]
    fn recognizes_bilibili_hosts_for_redirects() {
        assert!(is_bilibili_host(
            &reqwest::Url::parse("https://api.bilibili.com/x").unwrap()
        ));
        assert!(is_bilibili_host(
            &reqwest::Url::parse("https://www.bilibili.com/").unwrap()
        ));
        assert!(is_bilibili_host(
            &reqwest::Url::parse("https://passport.biligame.com/").unwrap()
        ));
        assert!(!is_bilibili_host(
            &reqwest::Url::parse("https://evilbilibili.com/").unwrap()
        ));
        assert!(!is_bilibili_host(
            &reqwest::Url::parse("https://example.com/").unwrap()
        ));
    }

    #[test]
    fn detects_wbi_failures() {
        assert!(is_wbi_failure(
            &serde_json::json!({"code": -352, "message": "risk"})
        ));
        assert!(is_wbi_failure(
            &serde_json::json!({"code": -403, "message": "denied"})
        ));
        assert!(is_wbi_failure(
            &serde_json::json!({"code": 0, "data": {"v_voucher": "voucher_x"}})
        ));
        assert!(!is_wbi_failure(
            &serde_json::json!({"code": -101, "message": "not logged in"})
        ));
        assert!(!is_wbi_failure(
            &serde_json::json!({"code": 0, "data": {"list": []}})
        ));
    }
}
