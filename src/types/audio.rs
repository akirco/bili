use serde::{Deserialize, Serialize};

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
