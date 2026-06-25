use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Get comment list (paginated)
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type (1=video, 12=article, 17=post, etc.)
    /// * `oid` - Target ID (avid for videos)
    /// * `sort` - Sort order: 0=by time, 1=by likes, 2=by replies (default 0)
    /// * `pn` - Page number (default 1)
    /// * `ps` - Items per page, max 20 (default 20)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing comment list and pagination info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn comment_list(
        &self,
        comment_type: i64,
        oid: i64,
        sort: Option<i64>,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let sort_str = sort.unwrap_or(0).to_string();
        let pn_str = pn.unwrap_or(1).to_string();
        let ps_str = ps.unwrap_or(20).to_string();

        self.get_raw(
            "https://api.bilibili.com/x/v2/reply/main",
            &[
                ("type", &comment_type.to_string()),
                ("oid", &oid.to_string()),
                ("sort", &sort_str),
                ("pn", &pn_str),
                ("ps", &ps_str),
            ],
        )
        .await
    }

    /// Get comment list (lazy load, WBI-signed)
    ///
    /// Uses WBI signature for authentication, supports lazy loading mode.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `mode` - Display mode: 0=by popularity only, 1=popularity+time, 2=by time only
    /// * `pagination_str` - Pagination info (obtained from previous response's data.cursor.pagination_reply.next_offset)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing comment list and cursor info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn comment_list_wbi(
        &self,
        comment_type: i64,
        oid: i64,
        mode: Option<i64>,
        pagination_str: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("type".to_string(), comment_type.to_string());
        params.insert("oid".to_string(), oid.to_string());
        if let Some(m) = mode {
            params.insert("mode".to_string(), m.to_string());
        }
        if let Some(p) = pagination_str {
            params.insert("pagination_str".to_string(), p.to_string());
        }

        self.wbi_get_raw("https://api.bilibili.com/x/v2/reply/wbi/main", params)
            .await
    }

    /// Post a comment
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `message` - Comment text (max 1000 characters)
    /// * `root` - Root comment rpid (used when replying to level 2+ comments)
    /// * `parent` - Parent comment rpid
    /// * `plat` - Platform: 1=web, 2=Android, 3=iOS, 4=WP
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the comment post result
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::CsrfNotFound` - CSRF token not found
    pub async fn comment_add(
        &self,
        comment_type: i64,
        oid: i64,
        message: &str,
        root: Option<i64>,
        parent: Option<i64>,
        plat: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("message".to_string(), message.to_string());
        if let Some(r) = root {
            form.insert("root".to_string(), r.to_string());
        }
        if let Some(p) = parent {
            form.insert("parent".to_string(), p.to_string());
        }
        if let Some(p) = plat {
            form.insert("plat".to_string(), p.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        let result: Option<serde_json::Value> = self
            .post("https://api.bilibili.com/x/v2/reply/add", &form)
            .await?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    /// Like a comment
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `rpid` - Target comment's rpid
    /// * `action` - Action: 0=unlike, 1=like (default 1)
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
    pub async fn comment_like(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("rpid".to_string(), rpid.to_string());
        if let Some(a) = action {
            form.insert("action".to_string(), a.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/action", &form)
            .await?;
        Ok(())
    }

    /// Dislike a comment
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `rpid` - Target comment's rpid
    /// * `action` - Action: 0=remove dislike, 1=dislike (default 1)
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
    pub async fn comment_hate(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("rpid".to_string(), rpid.to_string());
        if let Some(a) = action {
            form.insert("action".to_string(), a.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/hate", &form)
            .await?;
        Ok(())
    }

    /// Delete a comment
    ///
    /// Can only delete your own comments.
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `rpid` - Target comment's rpid to delete
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
    pub async fn comment_delete(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("rpid".to_string(), rpid.to_string());
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/del", &form)
            .await?;
        Ok(())
    }

    /// Pin or unpin a comment (admin only)
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `rpid` - Target comment's rpid
    /// * `action` - Action: 0=unpin, 1=pin (default 1)
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
    pub async fn comment_top(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("rpid".to_string(), rpid.to_string());
        if let Some(a) = action {
            form.insert("action".to_string(), a.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/top", &form)
            .await?;
        Ok(())
    }

    /// Report a comment
    ///
    /// # Arguments
    ///
    /// * `comment_type` - Comment area type
    /// * `oid` - Target ID
    /// * `rpid` - Target comment's rpid
    /// * `reason` - Report reason:
    ///   * 0=other, 1=spam, 2=pornography, 3=flooding, 4=provocation
    ///   * 5=spoiler, 6=politics, 7=personal attack, 8=irrelevant content
    ///   * 9=illegal, 10=vulgar, 11=illegal website, 12=gambling fraud
    ///   * 13=misinformation, 14=incitement, 15=privacy violation
    ///   * 16=floor-grabbing, 17=harmful to minors
    /// * `content` - Additional report notes (used when reason=0)
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
    pub async fn comment_report(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        reason: i64,
        content: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("type".to_string(), comment_type.to_string());
        form.insert("oid".to_string(), oid.to_string());
        form.insert("rpid".to_string(), rpid.to_string());
        form.insert("reason".to_string(), reason.to_string());
        if let Some(c) = content {
            form.insert("content".to_string(), c.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/report", &form)
            .await?;
        Ok(())
    }
}
