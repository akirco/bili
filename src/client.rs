use crate::credentials::{Credentials, CredentialsManager};
use crate::error::BiliError;
use crate::wbi::{WbiCache, signed_params};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[derive(Debug, serde::Deserialize)]
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
    pub creds: CredentialsManager,
    pub wbi_cache: Arc<RwLock<Option<WbiCache>>>,
}

/// Helper for building URL query parameters with minimal allocations.
///
/// Can be constructed from slices, Vecs, or HashMaps.
pub struct Params<'a> {
    items: Cow<'a, [(Cow<'a, str>, Cow<'a, str>)]>,
}

/// Helper for building form data (POST body).
///
/// Fluent API for building form data with minimal boilerplate.
pub struct FormBuilder {
    inner: HashMap<String, String>,
}

impl FormBuilder {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn push<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.inner.insert(key.into(), value.into());
        self
    }

    pub fn push_opt<K, V>(mut self, key: K, value: Option<V>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        if let Some(v) = value {
            self.inner.insert(key.into(), v.into());
        }
        self
    }

    pub fn csrf(self, token: String) -> Self {
        self.push("csrf", token)
    }

    pub fn build(self) -> HashMap<String, String> {
        self.inner
    }
}

impl Default for FormBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Params<'a> {
    pub fn new() -> Self {
        Self {
            items: Cow::Owned(Vec::new()),
        }
    }

    pub fn push<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        match self.items {
            Cow::Owned(ref mut vec) => vec.push((key.into(), value.into())),
            Cow::Borrowed(_) => {
                let mut vec: Vec<(Cow<'a, str>, Cow<'a, str>)> = self.items.to_vec();
                vec.push((key.into(), value.into()));
                self.items = Cow::Owned(vec);
            }
        }
    }

    pub fn as_query(&self) -> Vec<(&str, &str)> {
        self.items
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect()
    }
}

impl<'a> Default for Params<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<HashMap<String, String>> for Params<'a> {
    fn from(map: HashMap<String, String>) -> Self {
        let items: Vec<(Cow<'a, str>, Cow<'a, str>)> = map
            .into_iter()
            .map(|(k, v)| (Cow::Owned(k), Cow::Owned(v)))
            .collect();
        Self {
            items: Cow::Owned(items),
        }
    }
}

impl<'a> From<Vec<(String, String)>> for Params<'a> {
    fn from(vec: Vec<(String, String)>) -> Self {
        let items: Vec<(Cow<'a, str>, Cow<'a, str>)> = vec
            .into_iter()
            .map(|(k, v)| (Cow::Owned(k), Cow::Owned(v)))
            .collect();
        Self {
            items: Cow::Owned(items),
        }
    }
}

impl<'a> From<&'a [(&'a str, &'a str)]> for Params<'a> {
    fn from(slice: &'a [(&'a str, &'a str)]) -> Self {
        let items: Vec<(Cow<'a, str>, Cow<'a, str>)> = slice
            .iter()
            .map(|(k, v)| (Cow::Borrowed(*k), Cow::Borrowed(*v)))
            .collect();
        Self {
            items: Cow::Owned(items),
        }
    }
}

impl BiliClient {
    pub fn new() -> Result<Self, BiliError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );
        headers.insert(
            "Referer",
            HeaderValue::from_static("https://www.bilibili.com"),
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        Ok(BiliClient {
            inner: Arc::new(BiliClientInner {
                http,
                creds: CredentialsManager::new(),
                wbi_cache: Arc::new(RwLock::new(None)),
            }),
        })
    }

    pub async fn set_cookies(&self, sessdata: &str, bili_jct: &str, buvid3: &str) {
        self.inner.creds.set(sessdata, bili_jct, buvid3).await;
    }

    pub async fn csrf(&self) -> Result<String, BiliError> {
        self.inner.creds.csrf().await
    }

    pub async fn export_cookies(&self) -> Credentials {
        self.inner.creds.export().await
    }

    pub async fn import_cookies_from(&self, creds: &Credentials) {
        self.inner.creds.import_from(creds).await;
    }

    pub async fn import_cookies(
        &self,
        sessdata: Option<String>,
        bili_jct: Option<String>,
        buvid3: Option<String>,
        dedeuserid: Option<String>,
    ) {
        self.inner
            .creds
            .import(sessdata, bili_jct, buvid3, dedeuserid)
            .await;
    }

    pub(crate) async fn get_wbi_keys(&self) -> Result<(String, String), BiliError> {
        {
            let cache = self.inner.wbi_cache.read().await;
            if let Some(ref c) = *cache
                && c.is_valid()
            {
                return Ok((c.img_key.clone(), c.sub_key.clone()));
            }
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

        *cache = Some(WbiCache::new(img_key_owned.clone(), sub_key_owned.clone()));

        Ok((img_key_owned, sub_key_owned))
    }

    pub(crate) async fn wbi_get_raw(
        &self,
        url: &str,
        params: HashMap<String, String>,
    ) -> Result<Value, BiliError> {
        let (img_key, sub_key) = self.get_wbi_keys().await?;
        let signed = signed_params(params, &img_key, &sub_key);
        self.get_raw(url, Params::from(signed)).await
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params<'_>,
    ) -> Result<T, BiliError> {
        let mut req = self.inner.http.get(url).query(&params.as_query());
        if let Some(cookie) = self.inner.creds.build_cookie_header().await {
            req = req.header(COOKIE, cookie);
        }
        let resp = req.send().await?;

        let raw: BiliResponse<T> = resp.json().await?;
        if raw.code != 0 {
            return Err(BiliError::Api {
                code: raw.code,
                message: raw.message,
            });
        }
        raw.data.ok_or(BiliError::Api {
            code: -1,
            message: "No data field".into(),
        })
    }

    pub async fn get_raw(&self, url: &str, params: Params<'_>) -> Result<Value, BiliError> {
        let mut req = self.inner.http.get(url).query(&params.as_query());
        if let Some(cookie) = self.inner.creds.build_cookie_header().await {
            req = req.header(COOKIE, cookie);
        }
        let resp = req.send().await?;
        Ok(resp.json().await?)
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &HashMap<String, String>,
    ) -> Result<Option<T>, BiliError> {
        let mut req = self.inner.http.post(url).form(form);
        if let Some(cookie) = self.inner.creds.build_cookie_header().await {
            req = req.header(COOKIE, cookie);
        }
        let resp = req.send().await?;
        let raw: BiliResponse<T> = resp.json().await?;
        if raw.code != 0 {
            return Err(BiliError::Api {
                code: raw.code,
                message: raw.message,
            });
        }
        Ok(raw.data)
    }

    pub async fn resolve_bvid(&self, bvid: &str) -> Result<(i64, i64), BiliError> {
        let value = self
            .get_raw("https://api.bilibili.com/x/web-interface/view", {
                let mut params = Params::new();
                params.push("bvid", bvid);
                params
            })
            .await?;
        let aid = value["data"]["aid"]
            .as_i64()
            .ok_or_else(|| BiliError::Parse("aid missing".into()))?;
        let cid = value["data"]["cid"]
            .as_i64()
            .ok_or_else(|| BiliError::Parse("cid missing".into()))?;
        Ok((aid, cid))
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
