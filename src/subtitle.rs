use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Get AI subtitle (video smart summary)
    ///
    /// Calls the Bilibili AI subtitle API to get the AI-generated subtitle/summary for a video.
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID
    /// * `bvid` - Video BV ID
    /// * `cid` - Video segment cid
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing AI subtitle content
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn ai_subtitle(
        &self,
        aid: i64,
        bvid: &str,
        cid: i64,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("aid".to_string(), aid.to_string());
        params.insert("bvid".to_string(), bvid.to_string());
        params.insert("cid".to_string(), cid.to_string());

        self.wbi_get_raw(
            "https://api.bilibili.com/x/web-interface/view/conclusion/get",
            params,
        )
        .await
    }
}
