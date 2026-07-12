use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct DanmakuSendResult {
    pub action: String,
    pub dmid: i64,
    #[serde(rename = "dmid_str")]
    pub dmid_str: String,
    pub visible: bool,
}

/// A single danmaku element parsed from protobuf `seg.so`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct DanmakuElem {
    pub id: i64,
    /// Playback time in milliseconds
    pub progress: i64,
    /// Danmaku mode: 1-3: scroll, 4: bottom, 5: top, 7: advanced, 9: BAS
    pub mode: i32,
    /// Font size: 18: small, 25: standard, 36: large
    pub fontsize: i32,
    /// RGB color as a 24-bit integer (e.g. 0xFFFFFF for white)
    pub color: u32,
    /// Hash of the sender's UID
    pub mid_hash: u64,
    /// Danmaku text content
    pub content: String,
    /// Weight for ranking/filtering
    pub weight: i64,
    /// Action string
    pub action: String,
    /// Pool: 0: normal, 1: subtitle, 2: special
    pub pool: i64,
    /// ID as string
    pub id_str: String,
}

/// Legacy snapshot result (kept for backward compatibility with `/x/v2/dm/ajax`)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DanmakuSnapshotItem {
    pub pos: i64,
    pub row: i64,
    pub mode: i64,
    pub text: String,
    pub color: String,
    pub dmid: i64,
    pub weight: i64,
    pub ctime: i64,
    pub mid_hash: String,
    pub is_sub: bool,
    pub opacity: f64,
    pub progress: i64,
    pub weight_rank: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DanmakuSnapshotResult {
    pub status: i64,
    pub aid: i64,
    pub cid: i64,
    pub title: String,
    pub danmaku_count: i64,
    pub view_count: i64,
    pub max_row: i64,
    pub max_pos: i64,
    pub page: i64,
    pub pages: i64,
    pub items: Vec<DanmakuSnapshotItem>,
}
