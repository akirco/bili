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

    /// 获取音频播放地址
    ///
    /// `GET https://www.bilibili.com/audio/music-service-c/web/url`
    ///
    /// - `sid`: 音频 sid
    pub async fn stream_url(&self, sid: i64) -> Result<BiliResponse<AudioStreamUrl>, BiliError> {
        let params = Params::new()
            .push("sid", sid.to_string())
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
