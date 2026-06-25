use crate::credentials::CredentialsManager;
use crate::error::BiliError;
use crate::wbi::{WbiCache, signed_params};
use reqwest::header::{COOKIE, HeaderMap, HeaderValue, USER_AGENT};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
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
    pub http: reqwest::Client,
    pub creds: CredentialsManager,
    pub wbi_cache: Arc<RwLock<Option<WbiCache>>>,
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
            .cookie_store(true)
            .build()?;

        Ok(BiliClient {
            http,
            creds: CredentialsManager::new(),
            wbi_cache: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn set_cookies(&self, sessdata: &str, bili_jct: &str, buvid3: &str) {
        self.creds.set(sessdata, bili_jct, buvid3).await;
    }

    pub async fn csrf(&self) -> Result<String, BiliError> {
        self.creds.csrf().await
    }

    pub async fn export_cookies(
        &self,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        self.creds.export().await
    }

    pub async fn import_cookies(
        &self,
        sessdata: Option<String>,
        bili_jct: Option<String>,
        buvid3: Option<String>,
        dedeuserid: Option<String>,
    ) {
        self.creds
            .import(sessdata, bili_jct, buvid3, dedeuserid)
            .await;
    }

    pub async fn get_wbi_keys(&self) -> Result<(String, String), BiliError> {
        {
            let cache = self.wbi_cache.read().await;
            if let Some(ref c) = *cache
                && c.is_valid()
            {
                return Ok((c.img_key.clone(), c.sub_key.clone()));
            }
        }

        let value = self
            .get_raw("https://api.bilibili.com/x/web-interface/nav", &[])
            .await?;

        let img_url = value["data"]["wbi_img"]["img_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.img_url missing".to_string()))?;
        let sub_url = value["data"]["wbi_img"]["sub_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.sub_url missing".to_string()))?;

        let img_key = img_url
            .split('/')
            .next_back()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid img_url format".to_string()))?;

        let sub_key = sub_url
            .split('/')
            .next_back()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid sub_url format".to_string()))?;

        let new_cache = WbiCache::new(img_key.to_string(), sub_key.to_string());
        {
            let mut cache = self.wbi_cache.write().await;
            *cache = Some(new_cache);
        }

        Ok((img_key.to_string(), sub_key.to_string()))
    }

    pub(crate) async fn wbi_get_raw(
        &self,
        url: &str,
        params: HashMap<String, String>,
    ) -> Result<Value, BiliError> {
        let (img_key, sub_key) = self.get_wbi_keys().await?;
        let signed = signed_params(params, &img_key, &sub_key);

        let url_params: Vec<(&str, &str)> = signed
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.get_raw(url, &url_params).await
    }

    pub(crate) async fn wbi_post<T: DeserializeOwned>(
        &self,
        url: &str,
        params: HashMap<String, String>,
    ) -> Result<Option<T>, BiliError> {
        let (img_key, sub_key) = self.get_wbi_keys().await?;
        let signed = signed_params(params, &img_key, &sub_key);

        let w_rid = signed.get("w_rid").unwrap();
        let wts = signed.get("wts").unwrap();

        let url = format!("{}?w_rid={}&wts={}", url, w_rid, wts);

        let mut form = signed;
        form.remove("w_rid");
        form.remove("wts");

        self.post(&url, &form).await
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<T, BiliError> {
        let mut req = self.http.get(url).query(params);
        if let Some(cookie) = self.creds.build_cookie_header().await {
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

    pub async fn get_raw(&self, url: &str, params: &[(&str, &str)]) -> Result<Value, BiliError> {
        let mut req = self.http.get(url).query(params);
        if let Some(cookie) = self.creds.build_cookie_header().await {
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
        let mut req = self.http.post(url).form(form);
        if let Some(cookie) = self.creds.build_cookie_header().await {
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
}
