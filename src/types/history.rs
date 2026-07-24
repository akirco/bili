use serde::{Deserialize, Serialize};

use super::video::{VideoDimension, VideoOwner, VideoPage, VideoRights, VideoStat};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryItem {
    pub title: String,
    pub long_title: String,
    pub cover: String,
    pub covers: Option<Vec<String>>,
    pub uri: String,
    pub history: HistoryItemHistory,
    pub videos: i64,
    pub author_name: String,
    pub author_face: String,
    pub author_mid: i64,
    pub view_at: i64,
    pub progress: i64,
    pub badge: String,
    pub show_title: String,
    pub duration: i64,
    pub current: String,
    pub total: i64,
    pub new_desc: String,
    pub is_finish: i64,
    pub is_fav: i64,
    pub kid: i64,
    pub tag_name: String,
    pub live_status: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryItemHistory {
    pub oid: i64,
    pub epid: i64,
    pub bvid: String,
    pub page: i64,
    pub cid: i64,
    pub part: String,
    pub business: String,
    pub dt: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryTab {
    #[serde(rename = "type")]
    pub tab_type: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryCursor {
    pub max: i64,
    pub view_at: i64,
    pub business: String,
    pub ps: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryResult {
    pub list: Vec<HistoryItem>,
    pub has_more: bool,
    pub cursor: HistoryCursor,
    pub tab: Vec<HistoryTab>,
    pub business: String,
    pub ps: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ToviewItem {
    pub aid: i64,
    pub videos: i64,
    pub tid: i64,
    pub tname: String,
    pub copyright: i64,
    pub pic: String,
    pub title: String,
    pub pubdate: i64,
    pub ctime: i64,
    pub desc: String,
    pub state: i64,
    pub duration: i64,
    pub rights: VideoRights,
    pub owner: VideoOwner,
    pub stat: VideoStat,
    pub dynamic: String,
    pub dimension: VideoDimension,
    pub page: VideoPage,
    pub count: i64,
    pub cid: i64,
    pub progress: i64,
    pub add_at: i64,
    pub bvid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ToviewResult {
    pub count: i64,
    pub list: Vec<ToviewItem>,
}
