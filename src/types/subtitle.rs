use serde::{Deserialize, Serialize};

/// 视频 AI 总结结果（`view/conclusion/get` 的 `data`）。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitleConclusionResult {
    /// 总结状态：-1 不支持 / 0 有摘要 / 1 无摘要
    pub code: i64,
    pub model_result: SubtitleModelResult,
    /// 摘要 id
    pub stid: String,
    pub status: i64,
    pub like_num: i64,
    pub dislike_num: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitleModelResult {
    /// 0 没有摘要 / 1 仅总结 / 2 总结与提纲
    pub result_type: i64,
    /// 视频摘要
    pub summary: String,
    /// 分段提纲
    pub outline: Vec<SubtitleOutline>,
    /// AI 字幕（有结果时长度通常为 1）
    pub subtitle: Vec<SubtitleSection>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitleOutline {
    pub title: String,
    pub part_outline: Vec<SubtitlePartOutline>,
    /// 分段起始时间（秒）
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitlePartOutline {
    /// 要点起始时间（秒）
    pub timestamp: i64,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitleSection {
    pub part_subtitle: Vec<SubtitlePartSubtitle>,
    /// 字幕识别起始时间（秒）
    pub timestamp: i64,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SubtitlePartSubtitle {
    pub content: String,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
}
