use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct SubtitleClient {
    inner: Arc<BiliClientInner>,
}

impl SubtitleClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn list(&self, aid: i64, cid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("aid", aid.to_string());
        params.push("cid", cid.to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/view?aid=", params)
            .await
    }

    pub async fn summary(
        &self,
        aid: i64,
        cid: i64,
        segment_index: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("aid", aid.to_string());
        params.push("cid", cid.to_string());
        if let Some(s) = segment_index {
            params.push("segment_index", s.to_string());
        }
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/subtitle/summary", params)
            .await
    }
}