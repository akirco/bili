use crate::client::BiliClient;
use crate::error::BiliError;

impl BiliClient {
    /// Get Bilibili audio stream URL for an AU
    ///
    /// # Arguments
    ///
    /// * `sid` - Audio AU ID
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the direct audio URL and other info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn audio_stream_url(&self, sid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://www.bilibili.com/audio/music-service-c/web/url",
            &[
                ("sid", &sid.to_string()),
                ("privilege", "2"),
                ("quality", "2"),
            ],
        )
        .await
    }
}
