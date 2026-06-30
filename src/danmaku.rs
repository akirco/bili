use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct DanmakuClient {
    inner: Arc<BiliClientInner>,
}

impl DanmakuClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn snapshot(&self, aid: &str) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("aid", aid);
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/v2/dm/ajax", params)
            .await
    }

    pub async fn history_dates(
        &self,
        oid: i64,
        month: &str,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("type", "1");
        params.push("oid", oid.to_string());
        params.push("month", month);
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/v2/dm/history/index", params)
            .await
    }
}