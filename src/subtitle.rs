use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::{SubtitleConclusionResult, VideoSubtitle};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubtitleClient {
    client: BiliClient,
}

/// `data` of the player endpoint only carries `subtitle` among the fields we need.
#[derive(serde::Deserialize)]
struct PlayerSubtitleData {
    #[serde(default)]
    subtitle: VideoSubtitle,
}

impl SubtitleClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取视频字幕列表
    ///
    /// `GET https://api.bilibili.com/x/player/wbi/v2` (Wbi签名)
    ///
    /// 返回 `data.subtitle`；未登录或无字幕时字幕列表为空。
    ///
    /// - `aid`: 稿件 avid（与 bvid 任选一个）
    /// - `bvid`: 稿件 bvid（与 aid 任选一个）
    /// - `cid`: 视频 cid
    pub async fn list(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
    ) -> Result<BiliResponse<VideoSubtitle>, BiliError> {
        let params = video_params(aid, bvid, cid);
        let resp = self
            .client
            .wbi_get::<PlayerSubtitleData>("https://api.bilibili.com/x/player/wbi/v2", params)
            .await?;
        Ok(BiliResponse {
            code: resp.code,
            message: resp.message,
            ttl: resp.ttl,
            data: resp.data.map(|d| d.subtitle),
        })
    }

    /// 获取视频 AI 总结（摘要 + 提纲 + AI 字幕）
    ///
    /// `GET https://api.bilibili.com/x/web-interface/view/conclusion/get` (Wbi签名)
    ///
    /// - `aid`: 稿件 avid（与 bvid 任选一个）
    /// - `bvid`: 稿件 bvid（与 aid 任选一个）
    /// - `cid`: 视频 cid
    /// - `up_mid`: UP主 mid（可选）
    pub async fn summary(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
        up_mid: Option<i64>,
    ) -> Result<BiliResponse<SubtitleConclusionResult>, BiliError> {
        let params = video_params(aid, bvid, cid).push_opt_val("up_mid", up_mid);
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/view/conclusion/get",
                params,
            )
            .await
    }
}

fn video_params(aid: Option<i64>, bvid: Option<&str>, cid: i64) -> Params {
    Params::new()
        .push_opt_val("aid", aid)
        .push_opt("bvid", bvid)
        .push_val("cid", cid)
}
