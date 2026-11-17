use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct AudioClient {
    client: BiliClient,
}

impl AudioClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取音频播放地址（历史接口）
    ///
    /// `GET https://www.bilibili.com/audio/music-service-c/web/url`
    ///
    /// B 站音频区已停止投稿，但历史音频仍可通过该接口获取；付费音频只返回
    /// 试听片段。接口目前仍可用，因此未标记为 deprecated。
    ///
    /// - `sid`: 音频 sid
    pub async fn stream_url(&self, sid: i64) -> Result<BiliResponse<AudioStreamUrl>, BiliError> {
        let params = Params::new()
            .push_val("sid", sid)
            .push("privilege", "2")
            .push("quality", "2");
        self.client
            .get(
                "https://www.bilibili.com/audio/music-service-c/web/url",
                params,
            )
            .await
    }
}

/// 音频流地址（`audio/music-service-c/web/url` 的 `data`）。
///
/// 接口返回的音质字段名是 `type`，有效期是 `timeout`（秒），
/// 音频地址在 `cdns` 数组里（第一个通常为主地址，其余为备用）。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AudioStreamUrl {
    pub sid: i64,
    /// 音质标识：-1 试听 / 1 192K / 2 320K / 3 FLAC
    #[serde(rename = "type")]
    pub quality: i64,
    pub info: String,
    /// URL 有效时长，单位为秒（一般为 3 小时）
    pub timeout: i64,
    pub size: i64,
    pub cdns: Vec<String>,
    pub qualities: Option<Vec<AudioQuality>>,
    pub title: String,
    pub cover: String,
}

/// `AudioStreamUrl::qualities` 中的音质条目。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct AudioQuality {
    #[serde(rename = "type")]
    pub quality: i64,
    pub desc: String,
    pub size: i64,
    pub bps: String,
    pub tag: String,
    /// 0：不需要会员 / 1：需要会员
    pub require: i64,
    pub requiredesc: String,
}
