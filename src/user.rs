use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserClient {
    inner: Arc<BiliClientInner>,
}

impl UserClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn info(&self, mid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("mid".to_string(), mid.to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw("https://api.bilibili.com/x/space/wbi/acc/info", params)
            .await
    }

    pub async fn videos(
        &self,
        mid: i64,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("mid".to_string(), mid.to_string());
        params.insert("pn".to_string(), pn.unwrap_or(1).to_string());
        params.insert("ps".to_string(), ps.unwrap_or(30).to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw("https://api.bilibili.com/x/space/wbi/arc/search", params)
            .await
    }

    pub async fn stat(&self, mid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("mid", mid.to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/space/upstat", params)
            .await
    }

    pub async fn get_current_uid(&self) -> Result<i64, BiliError> {
        let uid = self.inner.creds.get_dedeuserid().await;
        uid.and_then(|s| s.parse().ok())
            .ok_or(BiliError::Parse("Cannot get current user UID".into()))
    }
}