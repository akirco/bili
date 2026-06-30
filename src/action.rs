
use crate::client::{BiliClientInner, FormBuilder};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct ActionClient {
    inner: Arc<BiliClientInner>,
}

impl ActionClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn coin_video(
        &self,
        aid: i64,
        multiply: Option<i64>,
        select_like: Option<bool>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("aid", aid.to_string())
            .push("multiply", multiply.unwrap_or(1).to_string())
            .push_opt("select_like", select_like.filter(|&v| v).map(|_| "1".to_string()))
            .csrf(csrf)
            .build();

        let data: Option<serde_json::Value> = crate::client::BiliClient { inner: self.inner.clone() }
            .post("https://api.bilibili.com/x/web-interface/coin/add", &form)
            .await?;
        data.ok_or(BiliError::Parse("coin response data missing".into()))
    }

    pub async fn has_coin(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = crate::client::Params::new();
        params.push("aid", aid.to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/archive/coins", params)
            .await
    }

    pub async fn like_video(&self, aid: i64, like: bool) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;
        let like_val = if like { "1" } else { "2" };

        let form = FormBuilder::new()
            .push("aid", aid.to_string())
            .push("like", like_val)
            .csrf(csrf)
            .build();

        let data: Option<serde_json::Value> = crate::client::BiliClient { inner: self.inner.clone() }
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like",
                &form,
            )
            .await?;
        Ok(data.unwrap_or(serde_json::Value::Null))
    }

    pub async fn has_like(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = crate::client::Params::new();
        params.push("aid", aid.to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/archive/has/like", params)
            .await
    }

    pub async fn triple(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("aid", aid.to_string())
            .csrf(csrf)
            .build();

        let data: Option<serde_json::Value> = crate::client::BiliClient { inner: self.inner.clone() }
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like/triple",
                &form,
            )
            .await?;
        data.ok_or(BiliError::Parse("triple response data missing".into()))
    }
}