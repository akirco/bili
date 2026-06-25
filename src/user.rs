use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Get user info
    ///
    /// Uses WBI signature to get detailed info for a specified user.
    ///
    /// # Arguments
    ///
    /// * `mid` - User ID
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing user nickname, avatar, signature, etc.
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn user_info(&self, mid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("mid".to_string(), mid.to_string());

        self.wbi_get_raw("https://api.bilibili.com/x/space/wbi/acc/info", params)
            .await
    }

    /// Get uploader's video list (paginated)
    ///
    /// Uses WBI signature to get all videos from a specified uploader.
    ///
    /// # Arguments
    ///
    /// * `mid` - Uploader's user ID
    /// * `pn` - Page number (default 1)
    /// * `ps` - Items per page (default 30)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing video list and pagination info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn user_videos(
        &self,
        mid: i64,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("mid".to_string(), mid.to_string());
        params.insert("pn".to_string(), pn.unwrap_or(1).to_string());
        params.insert("ps".to_string(), ps.unwrap_or(30).to_string());

        self.wbi_get_raw("https://api.bilibili.com/x/space/wbi/arc/search", params)
            .await
    }

    /// Get user statistics
    ///
    /// Get the user's following count, follower count, likes count and other statistics.
    ///
    /// # Arguments
    ///
    /// * `mid` - User ID
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing statistics
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn user_stat(&self, mid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/space/upstat",
            &[("mid", &mid.to_string())],
        )
        .await
    }

    /// Get current logged-in user's UID
    ///
    /// # Returns
    ///
    /// Returns the current logged-in user's UID
    ///
    /// # Errors
    ///
    /// * `BiliError::Parse` - Unable to parse UID
    pub async fn get_current_uid(&self) -> Result<i64, BiliError> {
        let uid = self.creds.get_dedeuserid().await;
        uid.and_then(|s| s.parse().ok())
            .ok_or(BiliError::Parse("Cannot get current user UID".to_string()))
    }
}
