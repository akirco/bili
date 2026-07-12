use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::{VideoHotResult, VideoInfo, VideoRecommendResult, VideoUrlResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct VideoClient {
    client: BiliClient,
}

impl VideoClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取视频详细信息 (web端)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/view`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    pub async fn info(&self, aid: Option<i64>, bvid: Option<&str>) -> Result<BiliResponse<VideoInfo>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push("aid", a.to_string());
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        self.client
            .get("https://api.bilibili.com/x/web-interface/view", params)
            .await
    }

    /// 获取视频流地址 (web端, Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/player/wbi/playurl`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    /// - `cid`: 视频 cid
    /// - `qn`: 视频清晰度 (默认80=1080P, DASH格式时无效)
    ///   - 16=360P, 32=480P, 64=720P, 74=720P60, 80=1080P, 112=1080P+, 116=1080P60, 120=4K, 125=HDR, 126=杜比视界, 127=8K
    /// - `fnval`: 视频流格式标识 (默认16=DASH)
    ///   - 1=MP4, 16=DASH, 64=HDR, 128=4K, 256=杜比音频, 512=杜比视界, 1024=8K, 2048=AV1, 4048=所有DASH
    /// - `fourk`: 是否允许4K (0=否, 1=是)
    pub async fn playurl(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
        qn: Option<i64>,
        fnval: Option<i64>,
        fourk: Option<i64>,
    ) -> Result<BiliResponse<VideoUrlResult>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push("avid", a.to_string());
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        params = params.push("cid", cid.to_string());
        params = params.push("qn", qn.unwrap_or(80).to_string());
        params = params.push("fnval", fnval.unwrap_or(16).to_string());
        params = params.push("fnver", "0");
        if let Some(f) = fourk {
            params = params.push("fourk", f.to_string());
        }

        self.client
            .wbi_get("https://api.bilibili.com/x/player/wbi/playurl", params)
            .await
    }

    /// 获取当前热门视频列表
    ///
    /// `GET https://api.bilibili.com/x/web-interface/popular`
    ///
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    pub async fn popular(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<VideoHotResult>, BiliError> {
        let params = Params::new()
            .push("pn", pn.unwrap_or(1).to_string())
            .push("ps", ps.unwrap_or(20).to_string());
        self.client
            .get("https://api.bilibili.com/x/web-interface/popular", params)
            .await
    }

    /// 获取相关推荐视频
    ///
    /// `GET https://api.bilibili.com/x/web-interface/archive/related`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    pub async fn related(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<VideoRecommendResult>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push("aid", a.to_string());
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/archive/related",
                params,
            )
            .await
    }

    /// 获取首页推荐视频
    ///
    /// `GET https://api.bilibili.com/x/web-interface/index/top/feed/rcmd`
    ///
    /// - `fresh_type`: 刷新类型 (默认3)
    /// - `ps`: 数量 (默认20)
    pub async fn rcmd(
        &self,
        ps: Option<i64>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let params = Params::new()
            .push("fresh_type", "3")
            .push("feed_version", "V8")
            .push("fresh_idx", "1")
            .push("ps", ps.unwrap_or(20).to_string());
        self.client
            .wbi_get("https://api.bilibili.com/x/web-interface/index/top/feed/rcmd", params)
            .await
    }

    /// 获取分区视频排行榜列表
    ///
    /// `GET https://api.bilibili.com/x/web-interface/ranking/v2`
    ///
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    /// - `rid`: 目标分区 tid (默认0=全站, 仅支持主分区)
    pub async fn hot(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
        rid: Option<&str>,
    ) -> Result<BiliResponse<VideoHotResult>, BiliError> {
        let params = Params::new()
            .push("pn", pn.unwrap_or(1).to_string())
            .push("ps", ps.unwrap_or(20).to_string())
            .push("rid", rid.unwrap_or("0"));
        self.client
            .get("https://api.bilibili.com/x/web-interface/ranking/v2", params)
            .await
    }
}
