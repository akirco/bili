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


