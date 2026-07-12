use serde::{Deserialize, Serialize};

use super::video::{VideoDimension, VideoOwner, VideoPage, VideoRights, VideoStat};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryItem {
    pub oid: i64,
    pub progress: i64,
    pub cid: i64,
    pub mid: i64,
    pub type_history: i64,
    pub business: String,
    pub src: String,
    pub title: String,
    pub cover: String,
    pub page: i64,
    pub part: String,
    pub duration: i64,
    pub aid: i64,
    pub bvid: String,
    pub epid: Option<i64>,
    pub season_id: Option<i64>,
    pub vt: Option<i64>,
    pub ctime: i64,
    pub mtime: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct HistoryResult {
    pub list: Vec<HistoryItem>,
    pub has_more: bool,
    pub max_dtime: i64,
    pub view_at: i64,
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
