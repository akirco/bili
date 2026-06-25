use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    pub async fn coin_video(
        &self,
        aid: i64,
        multiply: Option<i64>,
        select_like: Option<bool>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("aid".to_string(), aid.to_string());
        form.insert("multiply".to_string(), multiply.unwrap_or(1).to_string());
        if select_like.unwrap_or(false) {
            form.insert("select_like".to_string(), "1".to_string());
        }
        form.insert("csrf".to_string(), csrf);
        let data: Option<serde_json::Value> = self
            .post("https://api.bilibili.com/x/web-interface/coin/add", &form)
            .await?;
        data.ok_or(BiliError::Parse("coin response data missing".to_string()))
    }

    pub async fn has_coin(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/web-interface/archive/coins",
            &[("aid", &aid.to_string())],
        )
        .await
    }
}
