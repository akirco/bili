use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

fn to_str_slice<'a>(params: &'a [(&'a str, String)]) -> Vec<(&'a str, &'a str)> {
    params.iter().map(|(k, v)| (*k, v.as_str())).collect()
}

impl BiliClient {
    /// Get favorite folder metadata
    ///
    /// Returns detailed info about the favorite folder, including id, fid, title, intro, cover, cnt_info, etc.
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing favorite folder details
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn favorite_folder_info(
        &self,
        media_id: i64,
    ) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/info",
            &[("media_id", &media_id.to_string())],
        )
        .await
    }

    /// Get all favorite folders created by a specified user
    ///
    /// # Arguments
    ///
    /// * `up_mid` - User ID
    /// * `type` - Favorite folder type (default 2)
    /// * `rid` - Resource ID (optional, for querying the favorite status of a specific resource)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing all favorite folder info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn favorite_folder_list_all(
        &self,
        up_mid: i64,
        r#type: Option<i64>,
        rid: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Vec::new();
        params.push(("up_mid", up_mid.to_string()));
        let t = r#type.unwrap_or(2);
        params.push(("type", t.to_string()));
        if let Some(r) = rid {
            params.push(("rid", r.to_string()));
        }
        self.get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
            &to_str_slice(&params),
        )
        .await
    }

    /// Query user's collected favorite folders
    ///
    /// Get which favorite folders the user has collected.
    ///
    /// # Arguments
    ///
    /// * `up_mid` - User ID
    /// * `ps` - Items per page (optional)
    /// * `pn` - Page number (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the list of collected favorite folders
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn favorite_collected_list(
        &self,
        up_mid: i64,
        ps: Option<i64>,
        pn: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Vec::new();
        params.push(("up_mid", up_mid.to_string()));
        if let Some(p) = ps {
            params.push(("ps", p.to_string()));
        }
        if let Some(p) = pn {
            params.push(("pn", p.to_string()));
        }
        self.get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/collected/list",
            &to_str_slice(&params),
        )
        .await
    }

    /// Batch get content info for specified favorite IDs
    ///
    /// # Arguments
    ///
    /// * `resources` - Resource ID list in format "rid:type,rid:type" (e.g. "12345:2,67890:2")
    /// * `platform` - Platform identifier (optional)
    ///
    /// # Returns
    ///
    /// Returns a Vec of resource info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn favorite_resource_infos(
        &self,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Vec::new();
        params.push(("resources", resources.to_string()));
        if let Some(p) = platform {
            params.push(("platform", p.to_string()));
        }
        let mut resp: serde_json::Value = self
            .get_raw(
                "https://api.bilibili.com/x/v3/fav/resource/infos",
                &to_str_slice(&params),
            )
            .await?;
        Ok(serde_json::from_value(resp["data"].take())?)
    }

    /// Get favorite folder content list
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
    /// * `tid` - Category ID (optional, for filtering)
    /// * `keyword` - Search keyword (optional)
    /// * `type` - Resource type (optional)
    /// * `ps` - Items per page (optional)
    /// * `pn` - Page number (optional)
    /// * `platform` - Platform identifier (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing favorite folder content list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    #[allow(clippy::too_many_arguments)]
    pub async fn favorite_resource_list(
        &self,
        media_id: i64,
        tid: Option<i64>,
        keyword: Option<&str>,
        r#type: Option<i64>,
        ps: Option<i64>,
        pn: Option<i64>,
        platform: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Vec::new();
        params.push(("media_id", media_id.to_string()));
        if let Some(t) = tid {
            params.push(("tid", t.to_string()));
        }
        if let Some(k) = keyword {
            params.push(("keyword", k.to_string()));
        }
        if let Some(t) = r#type {
            params.push(("type", t.to_string()));
        }
        params.push(("ps", ps.unwrap_or(20).to_string()));
        if let Some(p) = pn {
            params.push(("pn", p.to_string()));
        }
        if let Some(p) = platform {
            params.push(("platform", p.to_string()));
        }
        self.get_raw(
            "https://api.bilibili.com/x/v3/fav/resource/list",
            &to_str_slice(&params),
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    /// Get all content IDs in a favorite folder
    ///
    /// Only returns the list of content IDs, without detailed info.
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
    /// * `platform` - Platform identifier (optional)
    ///
    /// # Returns
    ///
    /// Returns a Vec of content IDs
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn favorite_resource_ids(
        &self,
        media_id: i64,
        platform: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, BiliError> {
        let mut params = Vec::new();
        params.push(("media_id", media_id.to_string()));
        if let Some(p) = platform {
            params.push(("platform", p.to_string()));
        }
        let mut resp: serde_json::Value = self
            .get_raw(
                "https://api.bilibili.com/x/v3/fav/resource/ids",
                &to_str_slice(&params),
            )
            .await?;
        Ok(serde_json::from_value(resp["data"].take())?)
    }

    /// Create a new favorite folder
    ///
    /// # Arguments
    ///
    /// * `title` - Favorite folder title
    /// * `intro` - Favorite folder description (optional)
    /// * `privacy` - Privacy setting: 0=public, 1=private (optional)
    /// * `cover` - Cover image URL (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the newly created favorite folder info
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::CsrfNotFound` - CSRF token not found
    /// * `BiliError::Parse` - Response data missing
    pub async fn favorite_folder_add(
        &self,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("title".to_string(), title.to_string());
        if let Some(i) = intro {
            form.insert("intro".to_string(), i.to_string());
        }
        if let Some(p) = privacy {
            form.insert("privacy".to_string(), p.to_string());
        }
        if let Some(c) = cover {
            form.insert("cover".to_string(), c.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        let result: Option<serde_json::Value> = self
            .post("https://api.bilibili.com/x/v3/fav/folder/add", &form)
            .await?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    /// Edit favorite folder info
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
    /// * `title` - New title
    /// * `intro` - New description (optional)
    /// * `privacy` - New privacy setting (optional)
    /// * `cover` - New cover image URL (optional)
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
    pub async fn favorite_folder_edit(
        &self,
        media_id: i64,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("media_id".to_string(), media_id.to_string());
        form.insert("title".to_string(), title.to_string());
        if let Some(i) = intro {
            form.insert("intro".to_string(), i.to_string());
        }
        if let Some(p) = privacy {
            form.insert("privacy".to_string(), p.to_string());
        }
        if let Some(c) = cover {
            form.insert("cover".to_string(), c.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/folder/edit", &form)
            .await?;
        Ok(())
    }

    /// Delete favorite folders
    ///
    /// # Arguments
    ///
    /// * `media_ids` - Favorite folder IDs to delete, comma-separated
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
    pub async fn favorite_folder_del(&self, media_ids: &str) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("media_ids".to_string(), media_ids.to_string());
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/folder/del", &form)
            .await?;
        Ok(())
    }

    /// Batch copy content to another favorite folder
    ///
    /// # Arguments
    ///
    /// * `src_media_id` - Source favorite folder ID
    /// * `tar_media_id` - Target favorite folder ID
    /// * `mid` - User ID
    /// * `resources` - Resource ID list in format "rid:type,rid:type"
    /// * `platform` - Platform identifier (optional)
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
    pub async fn favorite_resource_copy(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("src_media_id".to_string(), src_media_id.to_string());
        form.insert("tar_media_id".to_string(), tar_media_id.to_string());
        form.insert("mid".to_string(), mid.to_string());
        form.insert("resources".to_string(), resources.to_string());
        if let Some(p) = platform {
            form.insert("platform".to_string(), p.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/copy", &form)
            .await?;
        Ok(())
    }

    /// Batch move content to another favorite folder
    ///
    /// # Arguments
    ///
    /// * `src_media_id` - Source favorite folder ID
    /// * `tar_media_id` - Target favorite folder ID
    /// * `mid` - User ID
    /// * `resources` - Resource ID list in format "rid:type,rid:type"
    /// * `platform` - Platform identifier (optional)
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
    pub async fn favorite_resource_move(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("src_media_id".to_string(), src_media_id.to_string());
        form.insert("tar_media_id".to_string(), tar_media_id.to_string());
        form.insert("mid".to_string(), mid.to_string());
        form.insert("resources".to_string(), resources.to_string());
        if let Some(p) = platform {
            form.insert("platform".to_string(), p.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/move", &form)
            .await?;
        Ok(())
    }

    /// Batch delete content from a favorite folder
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
    /// * `resources` - Resource ID list in format "rid:type,rid:type"
    /// * `platform` - Platform identifier (optional)
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
    pub async fn favorite_resource_batch_del(
        &self,
        media_id: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("media_id".to_string(), media_id.to_string());
        form.insert("resources".to_string(), resources.to_string());
        if let Some(p) = platform {
            form.insert("platform".to_string(), p.to_string());
        }
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>(
            "https://api.bilibili.com/x/v3/fav/resource/batch-del",
            &form,
        )
        .await?;
        Ok(())
    }

    /// Clear all invalid content from a favorite folder
    ///
    /// # Arguments
    ///
    /// * `media_id` - Favorite folder ID
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
    pub async fn favorite_resource_clean(&self, media_id: i64) -> Result<(), BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("media_id".to_string(), media_id.to_string());
        form.insert("csrf".to_string(), csrf);

        self.post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/clean", &form)
            .await?;
        Ok(())
    }

    /// Add a video to a favorite folder
    pub async fn favorite_add(
        &self,
        rid: i64,
        add_media_ids: i64,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("rid".to_string(), rid.to_string());
        form.insert("type".to_string(), "2".to_string());
        form.insert("add_media_ids".to_string(), add_media_ids.to_string());
        form.insert("csrf".to_string(), csrf);
        let data: Option<serde_json::Value> = self
            .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
            .await?;
        data.ok_or(BiliError::Parse(
            "favorite_add response data missing".to_string(),
        ))
    }

    /// Favorite or unfavorite a resource
    ///
    /// Can specify which favorite folders to add to and which to remove from.
    pub async fn fav_resource_deal(
        &self,
        rid: i64,
        r#type: i64,
        add_media_ids: Option<&str>,
        del_media_ids: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.csrf().await?;
        let mut form = HashMap::new();
        form.insert("rid".to_string(), rid.to_string());
        form.insert("type".to_string(), r#type.to_string());
        if let Some(v) = add_media_ids {
            form.insert("add_media_ids".to_string(), v.to_string());
        }
        if let Some(v) = del_media_ids {
            form.insert("del_media_ids".to_string(), v.to_string());
        }
        form.insert("csrf".to_string(), csrf);
        let data: Option<serde_json::Value> = self
            .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
            .await?;
        data.ok_or(BiliError::Parse(
            "fav_resource_deal response data missing".to_string(),
        ))
    }

    pub async fn has_favorite(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/v2/fav/video/favoured",
            &[("aid", &aid.to_string())],
        )
        .await
    }

    pub async fn favorite_list(&self, up_mid: i64) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
            &[("up_mid", &up_mid.to_string())],
        )
        .await
    }
}
