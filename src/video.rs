use crate::client::{BiliClientInner, Params};
use crate::error::BiliError;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct VideoClient {
    inner: Arc<BiliClientInner>,
}

impl VideoClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn info(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params.push("aid", a.to_string());
        }
        if let Some(b) = bvid {
            params.push("bvid", b);
        }
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/view", params)
            .await
    }

    pub async fn playurl(
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

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw("https://api.bilibili.com/x/player/wbi/playurl", params)
            .await
    }

    pub async fn popular(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("pn", pn.unwrap_or(1).to_string());
        params.push("ps", ps.unwrap_or(20).to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/popular", params)
            .await
    }

    pub async fn related(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params.push("aid", a.to_string());
        }
        if let Some(b) = bvid {
            params.push("bvid", b);
        }
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/archive/related", params)
            .await
    }

    pub async fn hot(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
        rid: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("pn", pn.unwrap_or(1).to_string());
        params.push("ps", ps.unwrap_or(20).to_string());
        params.push("rid", rid.unwrap_or(0).to_string());
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/ranking/v2", params)
            .await
    }

    pub async fn rank(&self, rid: &str) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("rid", rid);
        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/web-interface/ranking/v2", params)
            .await
    }
}