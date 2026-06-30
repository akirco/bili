use crate::client::{BiliClientInner, FormBuilder, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct HistoryClient {
    inner: Arc<BiliClientInner>,
}

impl HistoryClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn list(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("pn", pn.unwrap_or(1).to_string());
        params.push("ps", ps.unwrap_or(20).to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/history/cursor", params)
            .await
    }

    pub async fn delete(&self, aid: i64) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("aid", aid.to_string())
            .push("history_type", "archive")
            .push("type", "del")
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/web-interface/history/del", &form)
            .await?;
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", "clear")
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/web-interface/history/clear", &form)
            .await?;
        Ok(())
    }
}