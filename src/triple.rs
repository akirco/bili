use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    pub async fn triple(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("aid".to_string(), aid.to_string());
        form.insert("csrf".to_string(), csrf);
        let data: Option<serde_json::Value> = self
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like/triple",
                &form,
            )
            .await?;
        data.ok_or(BiliError::Parse("triple response data missing".to_string()))
    }
}
