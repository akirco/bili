use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct AudioClient {
    inner: Arc<BiliClientInner>,
}

impl AudioClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn stream_url(&self, sid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("sid", sid.to_string());
        params.push("privilege", "2");
        params.push("quality", "2");
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://www.bilibili.com/audio/music-service-c/web/url", params)
            .await
    }
}