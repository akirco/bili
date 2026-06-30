use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct LoginClient {
    inner: Arc<BiliClientInner>,
}

impl LoginClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn qrcode(&self) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("source", "main-fe-header");
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://passport.bilibili.com/x/passport-login/web/qrcode/generate", params)
            .await
    }

    pub async fn qrcode_status(&self, qrcode_key: &str) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("qrcode_key", qrcode_key);
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://passport.bilibili.com/x/passport-login/web/qrcode/poll", params)
            .await
    }
}