use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::AudioStreamUrl;
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
