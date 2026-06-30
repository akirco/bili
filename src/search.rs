use crate::client::BiliClientInner;
use crate::error::BiliError;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchClient {
    inner: Arc<BiliClientInner>,
}

impl SearchClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn all(
        &self,
        keyword: &str,
        page: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw(
                "https://api.bilibili.com/x/web-interface/wbi/search/all/v2",
                params,
            )
            .await
    }

    pub async fn query(
        &self,
        keyword: &str,
        page: Option<i64>,
        search_type: &str,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("search_type".to_string(), search_type.to_string());
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw(
                "https://api.bilibili.com/x/web-interface/wbi/search/type",
                params,
            )
            .await
    }

    pub async fn user(&self, keyword: &str) -> Result<Vec<serde_json::Value>, BiliError> {
        let mut params = HashMap::new();
        params.insert("search_type".to_string(), "bili_user".to_string());
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), "1".to_string());

        let resp = crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw(
                "https://api.bilibili.com/x/web-interface/wbi/search/type",
                params,
            )
            .await?;

        Ok(resp["data"]["result"]
            .as_array()
            .cloned()
            .unwrap_or_default())
    }

    pub async fn hot(&self, limit: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("platform".to_string(), "web".to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw(
                "https://api.bilibili.com/x/web-interface/wbi/search/square",
                params,
            )
            .await
    }

    pub async fn hot_word(&self) -> Result<serde_json::Value, BiliError> {
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://s.search.bilibili.com/main/hotword", crate::client::Params::new())
            .await
    }
}