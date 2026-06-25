use crate::client::BiliClient;
use crate::error::BiliError;

impl BiliClient {
    /// Get danmaku snapshot (latest 20)
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the latest 20 danmaku
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn get_danmaku_snapshot(&self, aid: &str) -> Result<serde_json::Value, BiliError> {
        self.get_raw("https://api.bilibili.com/x/v2/dm/ajax", &[("aid", aid)])
            .await
    }

    /// Query historical danmaku date list
    ///
    /// Get the list of dates that have historical danmaku for a given month.
    ///
    /// # Arguments
    ///
    /// * `oid` - Video cid
    /// * `month` - Month in "YYYY-MM" format (e.g. "2024-01")
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the list of dates with danmaku
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn get_danmaku_history_dates(
        &self,
        oid: i64,
        month: &str,
    ) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/v2/dm/history/index",
            &[("type", "1"), ("oid", &oid.to_string()), ("month", month)],
        )
        .await
    }
}
