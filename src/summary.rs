use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    pub async fn summary_set(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
        up_mid: Option<i64>,
        stid: &str,
        like_state: i64,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;

        let mut params = HashMap::new();
        if let Some(a) = aid {
            params.insert("aid".to_string(), a.to_string());
        }
        if let Some(b) = bvid {
            params.insert("bvid".to_string(), b.to_string());
        }
        params.insert("cid".to_string(), cid.to_string());
        if let Some(m) = up_mid {
            params.insert("up_mid".to_string(), m.to_string());
        }
        params.insert("stid".to_string(), stid.to_string());
        params.insert("like_state".to_string(), like_state.to_string());
        params.insert("csrf".to_string(), csrf);

        self.wbi_post::<serde_json::Value>(
            "https://api.bilibili.com/x/web-interface/view/conclusion/set",
            params,
        )
        .await
        .map(|v| v.unwrap_or(serde_json::Value::Null))
    }
}
