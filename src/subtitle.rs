use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::{SubtitleContentResult, VideoSubtitle};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubtitleClient {
    client: BiliClient,
}

impl SubtitleClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取视频字幕列表
    ///
    /// `GET https://api.bilibili.com/x/web-interface/view/subtitle` (Wbi签名)
    ///
    /// - `aid`: 视频 avid
    /// - `cid`: 视频 cid
    pub async fn list(&self, aid: i64, cid: i64) -> Result<BiliResponse<VideoSubtitle>, BiliError> {
        let params = Params::new()
            .push("aid", aid.to_string())
            .push("cid", cid.to_string());
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/view/subtitle",
                params,
            )
            .await
    }

    /// 获取视频字幕摘要
    ///
    /// `GET https://api.bilibili.com/x/web-interface/subtitle/summary`
    ///
    /// - `aid`: 视频 avid
    /// - `cid`: 视频 cid
    /// - `segment_index`: 分段索引
    pub async fn summary(
        &self,
        aid: i64,
        cid: i64,
        segment_index: Option<i64>,
    ) -> Result<BiliResponse<SubtitleContentResult>, BiliError> {
        let mut params = Params::new()
            .push("aid", aid.to_string())
            .push("cid", cid.to_string());
        if let Some(s) = segment_index {
            params = params.push("segment_index", s.to_string());
        }
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/subtitle/summary",
                params,
            )
            .await
    }
}
