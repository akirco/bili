use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Get history (cursor pagination)
    ///
    /// # Arguments
    ///
    /// * `max` - Max history ID for pagination (default 0)
    /// * `view_at` - View timestamp for pagination (default 0)
    /// * `business` - Business type filter (optional, e.g. "archive" for videos)
    /// * `ps` - Items per page (default 20)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing history list and cursor info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn history_cursor(
        &self,
        max: Option<i64>,
        view_at: Option<i64>,
        business: Option<&str>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let max_str = max.unwrap_or(0).to_string();
        let view_at_str = view_at.unwrap_or(0).to_string();
        let ps_str = ps.unwrap_or(20).to_string();
        let business_str = business.unwrap_or("").to_string();

        self.get_raw(
            "https://api.bilibili.com/x/web-interface/history/cursor",
            &[
                ("max", &max_str),
                ("view_at", &view_at_str),
                ("business", &business_str),
                ("ps", &ps_str),
            ],
        )
        .await
    }

    /// Get watch-later list
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the watch-later list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn toview_list(&self) -> Result<serde_json::Value, BiliError> {
        self.get_raw("https://api.bilibili.com/x/v2/history/toview/web", &[])
            .await
    }

    /// Add to watch-later
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::CsrfNotFound` - CSRF token not found
    pub async fn toview_add(&self, aid: i64) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("aid".to_string(), aid.to_string());
        form.insert("csrf".to_string(), csrf);
        self.post::<()>("https://api.bilibili.com/x/v2/history/toview/add", &form)
            .await?;
        Ok(())
    }
}
