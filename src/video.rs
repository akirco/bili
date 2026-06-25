use crate::client::BiliClient;
use crate::error::BiliError;
use std::collections::HashMap;

impl BiliClient {
    /// Get video details
    ///
    /// Get detailed video info by AV ID or BV ID, including title, description, category, uploader, etc.
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID (optional)
    /// * `bvid` - Video BV ID (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing video details
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn video_info(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params_raw: Vec<(String, String)> = Vec::new();
        if let Some(a) = aid {
            params_raw.push(("aid".to_string(), a.to_string()));
        }
        if let Some(b) = bvid {
            params_raw.push(("bvid".to_string(), b.to_string()));
        }

        let params: Vec<(&str, &str)> = params_raw
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.get_raw("https://api.bilibili.com/x/web-interface/view", &params)
            .await
    }

    /// Get video play URL
    ///
    /// Uses WBI signature to get video play URLs, including video stream and audio stream URLs.
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID (optional)
    /// * `bvid` - Video BV ID (optional)
    /// * `cid` - Video segment cid
    /// * `qn` - Video quality (optional):
    ///   * 127=8K, 120=4K, 116=1080P60, 112=1080P+, 80=1080P, 64=720P, 32=480P, 16=360P
    /// * `fnval` - Format flag (optional):
    ///   * 0=FLV, 1=MP4, 16=DASH, 64=HDR, 128=DOLBY VISION, 256=8K
    /// * `fourk` - Whether 4K is supported (optional, 1=support)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing video stream and audio stream URLs
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    /// * `BiliError::WbiSignFailed` - WBI sign failed
    pub async fn video_playurl(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
        qn: Option<i64>,
        fnval: Option<i64>,
        fourk: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        if let Some(a) = aid {
            params.insert("avid".to_string(), a.to_string());
        }
        if let Some(b) = bvid {
            params.insert("bvid".to_string(), b.to_string());
        }
        params.insert("cid".to_string(), cid.to_string());
        params.insert("qn".to_string(), qn.unwrap_or(80).to_string());
        params.insert("fnval".to_string(), fnval.unwrap_or(16).to_string());
        params.insert("fnver".to_string(), "0".to_string());
        if let Some(f) = fourk {
            params.insert("fourk".to_string(), f.to_string());
        }

        self.wbi_get_raw("https://api.bilibili.com/x/player/wbi/playurl", params)
            .await
    }

    /// Get popular video list
    ///
    /// # Arguments
    ///
    /// * `pn` - Page number (optional, default 1)
    /// * `ps` - Items per page (optional, default 20)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing popular video list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn popular_videos(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let pn_str = pn.map_or("1".to_string(), |v| v.to_string());
        let ps_str = ps.map_or("20".to_string(), |v| v.to_string());
        self.get_raw(
            "https://api.bilibili.com/x/web-interface/popular",
            &[("pn", &pn_str), ("ps", &ps_str)],
        )
        .await
    }

    /// Get related video recommendations
    ///
    /// Get related video recommendations based on video ID.
    ///
    /// # Arguments
    ///
    /// * `aid` - Video AV ID (optional)
    /// * `bvid` - Video BV ID (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing related video list
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn video_related(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params_raw: Vec<(String, String)> = Vec::new();
        if let Some(a) = aid {
            params_raw.push(("aid".to_string(), a.to_string()));
        }
        if let Some(b) = bvid {
            params_raw.push(("bvid".to_string(), b.to_string()));
        }

        let params: Vec<(&str, &str)> = params_raw
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.get_raw(
            "https://api.bilibili.com/x/web-interface/archive/related",
            &params,
        )
        .await
    }

    /// Get trending ranking videos
    ///
    /// # Arguments
    ///
    /// * `pn` - Page number (optional, default 1)
    /// * `ps` - Items per page (optional, default 20)
    /// * `rid` - Category ID (optional)
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing ranking videos
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn hot_videos(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
        rid: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let pn_str = pn.map_or("1".to_string(), |v| v.to_string());
        let ps_str = ps.map_or("20".to_string(), |v| v.to_string());
        let rid_str = rid.map_or("0".to_string(), |v| v.to_string());
        self.get_raw(
            "https://api.bilibili.com/x/web-interface/ranking/v2",
            &[("pn", &pn_str), ("ps", &ps_str), ("rid", &rid_str)],
        )
        .await
    }

    /// Get ranking videos (by category)
    ///
    /// # Arguments
    ///
    /// * `rid` - Category ID (default "all")
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn rank_videos(&self, rid: &str) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://api.bilibili.com/x/web-interface/ranking/v2",
            &[("rid", rid)],
        )
        .await
    }

    /// Resolve BV ID to get aid and first segment cid
    ///
    /// # Arguments
    ///
    /// * `bvid` - Video BV ID
    ///
    /// # Returns
    ///
    /// Returns (aid, cid) tuple
    ///
    /// # Errors
    ///
    /// * `BiliError::Parse` - Unable to parse aid/cid from response
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    /// * `BiliError::Api` - API returned error code
    pub async fn resolve_bvid(&self, bvid: &str) -> Result<(i64, i64), BiliError> {
        let info = self.video_info(None, Some(bvid)).await?;
        let aid = info["data"]["aid"]
            .as_i64()
            .ok_or_else(|| BiliError::Parse("missing aid".to_string()))?;
        let cid = info["data"]["cid"]
            .as_i64()
            .ok_or_else(|| BiliError::Parse("missing cid".to_string()))?;
        Ok((aid, cid))
    }
}
