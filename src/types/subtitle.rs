use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SubtitleContentItem {
    pub from: f64,
    pub to: f64,
    pub content: String,
    pub location: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct SubtitleContentResult {
    pub status: i64,
    pub body: Vec<SubtitleContentItem>,
}
