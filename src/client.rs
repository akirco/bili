use crate::cookie::{SharedCookieStore, default_cookie_path, new_shared};
use crate::error::BiliError;
use crate::wbi::{WbiCache, signed_params_with_mixin};
use reqwest::ClientBuilder as ReqwestClientBuilder;
use reqwest::RequestBuilder;
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde::ser::SerializeMap;
use std::collections::HashMap;
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

#[derive(Clone)]
pub struct BiliClientInner {
    pub http: reqwest::Client,
    /// Client without a total-request timeout, used for large downloads.
    pub download_http: reqwest::Client,
    pub cookie_store: SharedCookieStore,
    pub wbi_cache: Arc<RwLock<Option<WbiCache>>>,
    pub wbi_ttl: Duration,
}

/// Helper for building URL query parameters.
pub struct Params {
    items: Vec<(String, String)>,
}

/// Form data wrapper that preserves insertion order for POST bodies.
pub struct FormData(Vec<(String, String)>);

impl serde::Serialize for FormData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Helper for building form data (POST body).
pub struct FormBuilder {
    inner: Vec<(String, String)>,
}

impl FormBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    #[must_use]
    pub fn push<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.push((key.into(), value.into()));
        self
    }

    /// Push a key-value pair where the value implements `Display` (e.g. `i64`).
    #[must_use]
    pub fn push_val<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.inner.push((key.into(), value.to_string()));
        self
    }

    #[must_use]
    pub fn push_opt<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Some(v) = value {
            self.inner.push((key.into(), v.into()));
        }
        self
    }

    /// Push an optional key-value pair where the value implements `Display`.
    #[must_use]
    pub fn push_opt_val<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        if let Some(v) = value {
            self.inner.push((key.into(), v.to_string()));
        }
        self
    }

    #[must_use]
    pub fn csrf(self, token: String) -> Self {
        self.push("csrf", token)
    }

    #[must_use]
    pub fn build(self) -> FormData {
        FormData(self.inner)
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Params {
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    #[must_use]
    pub fn push<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.items.push((key.into(), value.into()));
        self
    }

    /// Push a key-value pair where the value implements `Display` (e.g. `i64`).
    #[must_use]
    pub fn push_val<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        self.items.push((key.into(), value.to_string()));
        self
    }

    #[must_use]
    pub fn push_opt<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Some(v) = value {
            self.items.push((key.into(), v.into()));
        }
        self
    }

    /// Push an optional key-value pair where the value implements `Display`.
    #[must_use]
    pub fn push_opt_val<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: std::fmt::Display,
    {
        if let Some(v) = value {
            self.items.push((key.into(), v.to_string()));
        }
        self
    }

    #[must_use]
    pub fn as_query(&self) -> Vec<(&str, &str)> {
        self.items
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    #[must_use]
    pub fn into_hashmap(self) -> HashMap<String, String> {
        self.items.into_iter().collect()
    }

    pub(crate) fn insert(&mut self, key: String, value: String) {
        self.items.push((key, value));
    }

    pub(crate) fn sort_by_key(&mut self) {
        self.items.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.items.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

impl Default for Params {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Params {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            items: map.into_iter().collect(),
        }
    }
}

impl From<Vec<(String, String)>> for Params {
    fn from(vec: Vec<(String, String)>) -> Self {
        Self { items: vec }
    }
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

    /// Flush pending cookie changes to disk.
    ///
    /// Normally this happens in `CookieStore::drop`, but the CLI calls
    /// `std::process::exit` on API errors, which skips destructors.
    pub fn flush_cookies(&self) {
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.flush();
        }
    }

    /// Parse cookies from a URL's query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::Parse`] if `url` is not a valid URL.
    pub fn parse_cookies(&self, url: &str) -> Result<(), BiliError> {
        let parsed = reqwest::Url::parse(url).map_err(|e| BiliError::Parse(e.to_string()))?;
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.set_from_url(&parsed);
        }
        Ok(())
    }

    /// Parse a `Cookie` header string (`SESSDATA=x; bili_jct=y`) into the store.
    pub fn parse_cookie_string(&self, cookies: &str) {
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.set_from_cookie_string(cookies);
        }
    }

    /// Get CSRF token from cookie store.
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::CsrfNotFound`] when not logged in or `bili_jct`
    /// is missing, or [`BiliError::CookieBuild`] if the store lock is poisoned.
    pub fn csrf(&self) -> Result<String, BiliError> {
        let token = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .csrf_token()
            .to_string();
        if token.is_empty() {
            Err(BiliError::CsrfNotFound)
        } else {
            Ok(token)
        }
    }

    /// Check if the user is logged in.
    #[must_use]
    pub fn is_logged_in(&self) -> bool {
        self.inner
            .cookie_store
            .lock()
            .is_ok_and(|s| s.is_logged_in())
    }

    /// Ensure the device cookies `buvid3`/`buvid4` exist.
    ///
    /// Bilibili's risk control rejects some endpoints with HTTP 412 when these
    /// cookies are missing. This is best-effort: if the endpoint fails, callers
    /// can still try their actual request.
    ///
    /// # Errors
    ///
    /// Propagates HTTP/parse errors from the fingerprint endpoint, or
    /// [`BiliError::CookieBuild`] if the cookie store lock is poisoned.
    pub async fn bootstrap_buvid(&self) -> Result<(), BiliError> {
        let has_buvid3 = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .has("buvid3");
        if has_buvid3 {
            return Ok(());
        }

        let raw = self.get_raw(BUVID_SPI_URL, Params::new()).await?;
        let Some((buvid3, buvid4)) = parse_buvid(&raw) else {
            return Ok(());
        };
        let mut store = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?;
        store.set_cookie("buvid3", buvid3);
        if let Some(buvid4) = buvid4 {
            store.set_cookie("buvid4", buvid4);
        }
        Ok(())
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    async fn try_wbi_cache(&self) -> Option<String> {
        let cache = self.inner.wbi_cache.read().await;
        cache
            .as_ref()
            .filter(|c| c.is_valid())
            .map(|c| c.mixin_key.clone())
    }

    /// Return the cached WBI `mixin_key`, fetching and caching it if needed.
    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn get_wbi_keys(&self) -> Result<String, BiliError> {
        if let Some(mixin_key) = self.try_wbi_cache().await {
            return Ok(mixin_key);
        }

        // Fetch without holding the lock so concurrent WBI requests are not
        // serialized behind this network call.
        let value = self
            .get_raw(
                "https://api.bilibili.com/x/web-interface/nav",
                Params::new(),
            )
            .await?;

        let img_url = value["data"]["wbi_img"]["img_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.img_url missing".into()))?;
        let sub_url = value["data"]["wbi_img"]["sub_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.sub_url missing".into()))?;

        let img_key = img_url
            .rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid img_url format".into()))?;

        let sub_key = sub_url
            .rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid sub_url format".into()))?;

        let cache =
            WbiCache::with_ttl(img_key.to_string(), sub_key.to_string(), self.inner.wbi_ttl);
        let mixin_key = cache.mixin_key.clone();

        // Double-check: another task may have refreshed the cache while we
        // were fetching. Prefer whichever value is already there.
        let mut guard = self.inner.wbi_cache.write().await;
        if let Some(ref existing) = *guard
            && existing.is_valid()
        {
            return Ok(existing.mixin_key.clone());
        }
        *guard = Some(cache);

        Ok(mixin_key)
    }

    fn with_cookie(
        &self,
        req: RequestBuilder,
        url: &reqwest::Url,
    ) -> Result<RequestBuilder, BiliError> {
        let cookie_header = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .build_cookie_header_for(url);
        if let Some(cookie) = cookie_header {
            Ok(req.header(COOKIE, cookie))
        } else {
            Ok(req)
        }
    }

    /// Send a request, optionally retrying transient failures.
    ///
    /// `retryable` must be `false` for non-idempotent requests (POST): a 5xx or
    /// timeout may mean the server already applied the action, and retrying
    /// would duplicate comments/danmaku/coins.
    async fn send_retry<F>(
        &self,
        url: &str,
        build: F,
        retryable: bool,
    ) -> Result<reqwest::Response, BiliError>
    where
        F: Fn() -> RequestBuilder,
    {
        let parsed = reqwest::Url::parse(url).map_err(|e| BiliError::Parse(e.to_string()))?;
        let max_attempts = if retryable { 3 } else { 1 };
        let mut delay = Duration::from_millis(100);
        for attempt in 0..max_attempts {
            let req = self.with_cookie(build(), &parsed)?;
            match req.send().await {
                Ok(resp) => {
                    // Auto-export Set-Cookie from the response URL's domain.
                    {
                        let headers = resp.headers().clone();
                        let response_url = resp.url().clone();
                        if let Ok(mut store) = self.inner.cookie_store.lock() {
                            store.update_from_response(&response_url, &headers);
                        }
                    }
                    let status = resp.status();
                    let transient = status.is_server_error()
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS;
                    if transient && retryable && attempt + 1 < max_attempts {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    // A cross-site redirect is stopped by the redirect policy;
                    // surface it instead of following without cookies or, worse,
                    // forwarding the Cookie header to an unrelated host.
                    if status.is_redirection() {
                        return Err(BiliError::Parse(format!(
                            "cross-site redirect blocked for cookie safety: {}",
                            resp.url()
                        )));
                    }
                    // Surface 4xx/5xx (e.g. 412 rate limiting, which returns
                    // HTML) as an HTTP error instead of a confusing JSON error.
                    if status == reqwest::StatusCode::PRECONDITION_FAILED
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    {
                        return Err(BiliError::RateLimited);
                    }
                    if !status.is_success() {
                        return Err(resp.error_for_status().unwrap_err().into());
                    }
                    return Ok(resp);
                }
                Err(e)
                    if retryable
                        && attempt + 1 < max_attempts
                        && (e.is_timeout() || e.is_connect()) =>
                {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
                Err(e) => return Err(e.into()),
            }
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    async fn invalidate_wbi_cache(&self) {
        *self.inner.wbi_cache.write().await = None;
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_get_raw(
        &self,
        url: &str,
        params: Params,
    ) -> Result<serde_json::Value, BiliError> {
        let mut mixin_key = self.get_wbi_keys().await?;
        for attempt in 0..2 {
            let signed = signed_params_with_mixin(&params, &mixin_key);
            let raw = self.get_raw(url, signed).await?;
            if attempt == 0 && is_wbi_failure(&raw) {
                // Keys rotate daily; a stale cached key makes Bilibili return
                // -352/-403 or a `v_voucher`. Refresh once and retry.
                self.invalidate_wbi_cache().await;
                mixin_key = self.get_wbi_keys().await?;
                continue;
            }
            return Ok(raw);
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let raw = self.wbi_get_raw(url, params).await?;
        if is_wbi_failure(&raw) {
            return Err(BiliError::RateLimited);
        }
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let resp = self
            .send_retry(
                url,
                || self.inner.http.get(url).query(&params.as_query()),
                true,
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_raw(&self, url: &str, params: Params) -> Result<serde_json::Value, BiliError> {
        let resp = self
            .send_retry(
                url,
                || self.inner.http.get(url).query(&params.as_query()),
                true,
            )
            .await?;
        Ok(resp.json().await?)
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_post_raw(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<serde_json::Value, BiliError> {
        let mut mixin_key = self.get_wbi_keys().await?;
        for attempt in 0..2 {
            let signed = signed_params_with_mixin(&params, &mixin_key);
            let resp = self
                .send_retry(
                    url,
                    || {
                        self.inner
                            .http
                            .post(url)
                            .query(&signed.as_query())
                            .form(form)
                    },
                    false,
                )
                .await?;
            let raw: serde_json::Value = resp.json().await?;
            if attempt == 0 && is_wbi_failure(&raw) {
                self.invalidate_wbi_cache().await;
                mixin_key = self.get_wbi_keys().await?;
                continue;
            }
            return Ok(raw);
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_post<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<BiliResponse<T>, BiliError> {
        let raw = self.wbi_post_raw(url, params, form).await?;
        if is_wbi_failure(&raw) {
            return Err(BiliError::RateLimited);
        }
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, BiliError> {
        let resp = self
            .send_retry(url, || self.inner.http.get(url), true)
            .await?;
        Ok(resp.bytes().await?.to_vec())
    }

    #[must_use]
    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// HTTP client for large file downloads.
    ///
    /// Unlike [`Self::http`] it has no total-request timeout (only a 60s read
    /// timeout), so slow multi-hundred-MB downloads are not cut off.
    #[must_use]
    pub fn download_http(&self) -> &reqwest::Client {
        &self.inner.download_http
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &FormData,
    ) -> Result<BiliResponse<T>, BiliError> {
        let resp = self
            .send_retry(url, || self.inner.http.post(url).form(form), false)
            .await?;
        Ok(resp.json().await?)
    }

    /// Resolve a video's identifiers (`aid`, first-part `cid`, `pages`) from a `bvid`.
    ///
    /// This is the convenience entry point used by the CLI: every command that
    /// needs an `aid` or a `cid` can start from a `bvid`.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_video_ids(&self, bvid: &str) -> Result<crate::types::VideoIds, BiliError> {
        let mut params = Params::new().push("bvid", bvid);
        self.fetch_video_ids(&mut params).await
    }

    /// Resolve a video's `(aid, cid)` from a `bvid`.
    ///
    /// `cid` is the first part's cid; use [`Self::resolve_video_ids`] to reach
    /// other parts of a multi-part video.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_bvid(&self, bvid: &str) -> Result<(i64, i64), BiliError> {
        let ids = self.resolve_video_ids(bvid).await?;
        Ok((ids.aid, ids.cid))
    }

    /// Resolve a video's `cid` from either an `aid` or a `bvid`.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_cid(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<i64, BiliError> {
        let mut params = Params::new();
        if let Some(aid) = aid {
            params = params.push_val("aid", aid);
        }
        if let Some(bvid) = bvid {
            params = params.push("bvid", bvid);
        }
        Ok(self.fetch_video_ids(&mut params).await?.cid)
    }

    async fn fetch_video_ids(
        &self,
        params: &mut Params,
    ) -> Result<crate::types::VideoIds, BiliError> {
        use crate::types::{VideoIds, VideoInfo};
        let endpoint = format!(
            "view?{}",
            params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        );
        let resp = self
            .get::<VideoInfo>(
                "https://api.bilibili.com/x/web-interface/view",
                std::mem::take(params),
            )
            .await?;
        let info = resp.data.ok_or(BiliError::NullData { endpoint })?;
        Ok(VideoIds {
            aid: info.aid,
            cid: info.cid,
            pages: info.pages,
        })
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
    fn params_new_is_empty() {
        let p = Params::new();
        assert!(p.iter().next().is_none());
    }

    #[test]
    fn params_push_chain() {
        let p = Params::new().push("a", "1").push("b", "2");
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], ("a", "1"));
        assert_eq!(items[1], ("b", "2"));
    }

    #[test]
    fn params_as_query() {
        let p = Params::new().push("key", "val");
        let q = p.as_query();
        assert_eq!(q, vec![("key", "val")]);
    }

    #[test]
    fn params_into_hashmap() {
        let p = Params::new().push("x", "1").push("y", "2");
        let map = p.into_hashmap();
        assert_eq!(map.get("x").unwrap(), "1");
        assert_eq!(map.get("y").unwrap(), "2");
    }

    #[test]
    fn params_from_hashmap() {
        let mut map = HashMap::new();
        map.insert("k".to_string(), "v".to_string());
        let p = Params::from(map);
        assert_eq!(p.iter().count(), 1);
    }

    #[test]
    fn params_from_vec() {
        let v = vec![("a".to_string(), "1".to_string())];
        let p = Params::from(v);
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("a", "1")]);
    }

    #[test]
    fn params_push_val() {
        let p = Params::new().push("a", "str").push_val("b", 42i64);
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("a", "str"), ("b", "42")]);
    }

    #[test]
    fn params_push_opt_val_none() {
        let p = Params::new().push("a", "1").push_opt_val("b", None::<i64>);
        assert_eq!(p.iter().count(), 1);
    }

    #[test]
    fn params_push_opt_val_some() {
        let p = Params::new().push_opt_val("b", Some(42i64));
        let items: Vec<_> = p.iter().collect();
        assert_eq!(items, vec![("b", "42")]);
    }

    #[test]
    fn form_builder_push() {
        let f = FormBuilder::new().push("type", "1").push("oid", "123");
        let data = f.build();
        assert_eq!(data.0.len(), 2);
        assert_eq!(data.0[0], ("type".to_string(), "1".to_string()));
        assert_eq!(data.0[1], ("oid".to_string(), "123".to_string()));
    }

    #[test]
    fn form_builder_push_opt_none() {
        let f = FormBuilder::new()
            .push("a", "1")
            .push_opt("b", None::<String>);
        let data = f.build();
        assert_eq!(data.0.len(), 1);
    }

    #[test]
    fn form_builder_push_opt_some() {
        let f = FormBuilder::new().push_opt("b", Some("val"));
        let data = f.build();
        assert_eq!(data.0, vec![("b".to_string(), "val".to_string())]);
    }

    #[test]
    fn form_builder_csrf() {
        let f = FormBuilder::new()
            .push("aid", "1")
            .csrf("token123".to_string());
        let data = f.build();
        assert_eq!(data.0.last().unwrap().0, "csrf");
        assert_eq!(data.0.last().unwrap().1, "token123");
    }

    #[test]
    fn form_data_serializes_as_map() {
        let f = FormBuilder::new().push("k", "v");
        let data = f.build();
        let json = serde_json::to_value(&data).unwrap();
        assert!(json.is_object());
        assert_eq!(json["k"], "v");
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
