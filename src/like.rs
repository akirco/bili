use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    pub async fn like_video(&self, aid: i64, like: bool) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let like_val = if like { "1" } else { "2" };
        let mut form = HashMap::new();
        form.insert("aid".to_string(), aid.to_string());
        form.insert("like".to_string(), like_val.to_string());
        form.insert("csrf".to_string(), csrf);
        let data: Option<serde_json::Value> = self
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like",
                &form,
            )
            .await?;
        Ok(data.unwrap_or(serde_json::Value::Null))
    }

    pub async fn has_like(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/web-interface/archive/has/like",
            &[("aid", &aid.to_string())],
        )
        .await
    }
}
