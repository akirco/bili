use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Comprehensive search
    ///
    /// Search videos, articles, users, and all other content.
    ///
    /// # Arguments
    ///
    /// * `keyword` - Search keyword
    /// * `page` - Page number (default 1)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing search results
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn search(
        &self,
        keyword: &str,
        page: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());

        self.wbi_get_raw(
            "https://api.bilibili.com/x/web-interface/wbi/search/all/v2",
            params,
        )
        .await
    }

    /// Search by type
    ///
    /// Search for content of a specified type.
    ///
    /// # Arguments
    ///
    /// * `keyword` - Search keyword
    /// * `page` - Page number (default 1)
    /// * `search_type` - Search type (video, bili_user, article, etc.)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing search results
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn search_keyword(
        &self,
        keyword: &str,
        page: Option<i64>,
        search_type: &str,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("search_type".to_string(), search_type.to_string());
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), page.unwrap_or(1).to_string());

        self.wbi_get_raw(
            "https://api.bilibili.com/x/web-interface/wbi/search/type",
            params,
        )
        .await
    }

    /// Search users
    ///
    /// Search for Bilibili users specifically.
    ///
    /// # Arguments
    ///
    /// * `keyword` - Search keyword
    ///
    /// # Returns
    ///
    /// Returns a Vec of users
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn search_user(&self, keyword: &str) -> Result<Vec<serde_json::Value>, BiliError> {
        let mut params = HashMap::new();
        params.insert("search_type".to_string(), "bili_user".to_string());
        params.insert("keyword".to_string(), keyword.to_string());
        params.insert("page".to_string(), "1".to_string());

        let resp = self
            .wbi_get_raw(
                "https://api.bilibili.com/x/web-interface/wbi/search/type",
                params,
            )
            .await?;

        Ok(resp["data"]["result"]
            .as_array()
            .cloned()
            .unwrap_or_default())
    }

    /// Get trending search list (Web WBI-signed)
    ///
    /// # Arguments
    ///
    /// * `limit` - Number of trending items
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing trending search list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn get_hot_search(&self, limit: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("limit".to_string(), limit.to_string());
        params.insert("platform".to_string(), "web".to_string());

        self.wbi_get_raw(
            "https://api.bilibili.com/x/web-interface/wbi/search/square",
            params,
        )
        .await
    }

    /// Get trending search words (Web)
    ///
    /// Get the list of trending words in the search box.
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing trending word list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn get_hot_word(&self) -> Result<serde_json::Value, BiliError> {
        self.get_raw("https://s.search.bilibili.com/main/hotword", &[])
            .await
    }
}
