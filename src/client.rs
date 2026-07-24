use crate::cookie::{SharedCookieStore, default_cookie_path, new_shared};
use crate::error::BiliError;
use crate::wbi::{WbiCache, signed_params};
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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct BiliResponse<T> {
    pub code: i64,
    pub message: String,
    pub ttl: Option<i64>,
    pub data: Option<T>,
}

#[derive(Clone)]
pub struct BiliClient {
    pub(crate) inner: Arc<BiliClientInner>,
}

#[derive(Clone)]
pub struct BiliClientInner {
    pub http: reqwest::Client,
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

    pub fn as_query(&self) -> Vec<(&str, &str)> {
        self.items
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

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
            wbi_ttl: Duration::from_secs(3600 * 6),
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
            .default_headers(headers)
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .build()?;

        Ok(BiliClient {
            inner: Arc::new(BiliClientInner {
                http,
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

    pub fn builder() -> BiliClientBuilder {
        BiliClientBuilder::default()
    }

    /// Parse cookies from a URL's query parameters.
    pub async fn parse_cookies(&self, url: &str) -> Result<(), BiliError> {
        let parsed = reqwest::Url::parse(url).map_err(|e| BiliError::Parse(e.to_string()))?;
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.set_from_url(&parsed);
        }
        Ok(())
    }

    /// Get CSRF token from cookie store.
    pub async fn csrf(&self) -> Result<String, BiliError> {
        self.inner
            .cookie_store
            .lock()
            .map(|s| s.csrf_token().to_string())
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))
    }

    /// Check if the user is logged in.
    pub async fn is_logged_in(&self) -> bool {
        self.inner
            .cookie_store
            .lock()
            .map(|s| s.is_logged_in())
            .unwrap_or(false)
    }

    async fn try_wbi_cache(&self) -> Option<(String, String)> {
        let cache = self.inner.wbi_cache.read().await;
        cache.as_ref().and_then(|c| {
            if c.is_valid() {
                Some((c.img_key.clone(), c.sub_key.clone()))
            } else {
                None
            }
        })
    }

    pub(crate) async fn get_wbi_keys(&self) -> Result<(String, String), BiliError> {
        if let Some(keys) = self.try_wbi_cache().await {
            return Ok(keys);
        }

        let mut cache = self.inner.wbi_cache.write().await;
        if let Some(ref c) = *cache
            && c.is_valid()
        {
            return Ok((c.img_key.clone(), c.sub_key.clone()));
        }

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

        let img_key_owned = img_key.to_string();
        let sub_key_owned = sub_key.to_string();

        *cache = Some(WbiCache::with_ttl(
            img_key_owned.clone(),
            sub_key_owned.clone(),
            self.inner.wbi_ttl,
        ));

        Ok((img_key_owned, sub_key_owned))
    }

    async fn with_cookie(&self, req: RequestBuilder) -> Result<RequestBuilder, BiliError> {
        let cookie_header = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .build_cookie_header();
        if let Some(cookie) = cookie_header {
            Ok(req.header(COOKIE, cookie))
        } else {
            Ok(req)
        }
    }

    /// Send a request with retry on transient failures.
    async fn send_retry<F>(&self, build: F) -> Result<reqwest::Response, BiliError>
    where
        F: Fn() -> RequestBuilder,
    {
        let mut delay = Duration::from_millis(100);
        for attempt in 0..3 {
            let req = self.with_cookie(build()).await?;
            match req.send().await {
                Ok(resp) => {
                    // Auto-export Set-Cookie from response
                    {
                        let headers = resp.headers().clone();
                        if let Ok(mut store) = self.inner.cookie_store.lock() {
                            store.update_from_response(&headers);
                        }
                    }
                    return Ok(resp);
                }
                Err(e) if attempt < 2 && (e.is_timeout() || e.is_connect()) => {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
                Err(e) => return Err(e.into()),
            }
        }
        unreachable!()
    }

    pub(crate) async fn wbi_get_raw(
        &self,
        url: &str,
        params: Params,
    ) -> Result<serde_json::Value, BiliError> {
        let (img_key, sub_key) = self.get_wbi_keys().await?;
        let signed = signed_params(params, &img_key, &sub_key);
        self.get_raw(url, signed).await
    }

    pub(crate) async fn wbi_get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let raw = self.wbi_get_raw(url, params).await?;
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let resp = self
            .send_retry(|| self.inner.http.get(url).query(&params.as_query()))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_raw(&self, url: &str, params: Params) -> Result<serde_json::Value, BiliError> {
        let resp = self
            .send_retry(|| self.inner.http.get(url).query(&params.as_query()))
            .await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn wbi_post_raw(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<serde_json::Value, BiliError> {
        let (img_key, sub_key) = self.get_wbi_keys().await?;
        let signed = signed_params(params, &img_key, &sub_key);
        let resp = self
            .send_retry(|| {
                self.inner
                    .http
                    .post(url)
                    .query(&signed.as_query())
                    .form(form)
            })
            .await?;
        Ok(resp.json().await?)
    }

    pub(crate) async fn wbi_post<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<BiliResponse<Option<T>>, BiliError> {
        let raw = self.wbi_post_raw(url, params, form).await?;
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, BiliError> {
        let resp = self.send_retry(|| self.inner.http.get(url)).await?;
        Ok(resp.bytes().await?.to_vec())
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &FormData,
    ) -> Result<BiliResponse<Option<T>>, BiliError> {
        let resp = self
            .send_retry(|| self.inner.http.post(url).form(form))
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn resolve_bvid(&self, bvid: &str) -> Result<(i64, i64), BiliError> {
        use crate::types::VideoInfo;
        let resp = self
            .get::<VideoInfo>(
                "https://api.bilibili.com/x/web-interface/view",
                Params::new().push("bvid", bvid),
            )
            .await?;
        let info = resp.data.ok_or_else(|| BiliError::NullData {
            endpoint: format!("view?bvid={}", bvid),
        })?;
        Ok((info.aid, info.cid))
    }

    #[cfg(feature = "video")]
    pub fn video(&self) -> crate::video::VideoClient {
        crate::video::VideoClient::new(self.inner.clone())
    }

    #[cfg(feature = "user")]
    pub fn user(&self) -> crate::user::UserClient {
        crate::user::UserClient::new(self.inner.clone())
    }

    #[cfg(feature = "search")]
    pub fn search(&self) -> crate::search::SearchClient {
        crate::search::SearchClient::new(self.inner.clone())
    }

    #[cfg(feature = "comment")]
    pub fn comment(&self) -> crate::comment::CommentClient {
        crate::comment::CommentClient::new(self.inner.clone())
    }

    #[cfg(feature = "fav")]
    pub fn fav(&self) -> crate::fav::FavClient {
        crate::fav::FavClient::new(self.inner.clone())
    }

    #[cfg(feature = "danmaku")]
    pub fn danmaku(&self) -> crate::danmaku::DanmakuClient {
        crate::danmaku::DanmakuClient::new(self.inner.clone())
    }

    #[cfg(feature = "audio")]
    pub fn audio(&self) -> crate::audio::AudioClient {
        crate::audio::AudioClient::new(self.inner.clone())
    }

    #[cfg(feature = "history")]
    pub fn history(&self) -> crate::history::HistoryClient {
        crate::history::HistoryClient::new(self.inner.clone())
    }

    #[cfg(feature = "login")]
    pub fn login(&self) -> crate::login::LoginClient {
        crate::login::LoginClient::new(self.inner.clone())
    }

    #[cfg(feature = "subtitle")]
    pub fn subtitle(&self) -> crate::subtitle::SubtitleClient {
        crate::subtitle::SubtitleClient::new(self.inner.clone())
    }

    #[cfg(feature = "action")]
    pub fn action(&self) -> crate::action::ActionClient {
        crate::action::ActionClient::new(self.inner.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(b.wbi_ttl, Duration::from_secs(3600 * 6));
    }

    #[test]
    fn client_builder_custom() {
        let client = BiliClientBuilder::default()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .wbi_ttl(Duration::from_secs(1800))
            .build()
            .unwrap();
        assert_eq!(client.inner.wbi_ttl, Duration::from_secs(1800));
    }
}
