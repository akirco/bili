use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavFolderUpper {
    pub mid: i64,
    pub name: String,
    pub face: String,
    pub followed: Option<bool>,
    pub vip_type: Option<i64>,
    pub vip_statue: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavFolderCntInfo {
    pub collect: i64,
    pub play: i64,
    pub thumb_up: i64,
    pub share: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavFolderInfo {
    pub id: i64,
    pub fid: i64,
    pub mid: i64,
    pub attr: i64,
    pub title: String,
    pub cover: Option<String>,
    pub upper: Option<FavFolderUpper>,
    pub cover_type: Option<i64>,
    pub cnt_info: Option<FavFolderCntInfo>,
    pub r#type: Option<i64>,
    pub intro: Option<String>,
    pub ctime: Option<i64>,
    pub mtime: Option<i64>,
    pub state: Option<i64>,
    pub fav_state: Option<i64>,
    pub like_state: Option<i64>,
    pub media_count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavFolderListResult {
    pub count: i64,
    pub list: Vec<FavFolderInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavResourceUpper {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavResourceCntInfo {
    pub collect: i64,
    pub play: i64,
    pub danmaku: i64,
    pub thumb_up: Option<i64>,
    pub share: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavResource {
    pub id: i64,
    pub r#type: i64,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: i64,
    pub duration: i64,
    pub upper: FavResourceUpper,
    pub attr: i64,
    pub cnt_info: FavResourceCntInfo,
    pub link: String,
    pub ctime: i64,
    pub pubtime: i64,
    pub fav_time: i64,
    pub bv_id: String,
    pub bvid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavResourceListResult {
    pub info: FavFolderInfo,
    pub medias: Vec<FavResource>,
    pub has_more: bool,
    pub ttl: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavResourceId {
    pub id: i64,
    pub r#type: i64,
    pub bv_id: String,
    pub bvid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavDealResult {
    pub prompt: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct FavHasFavoriteResult {
    pub count: i64,
    pub favoured: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct EmptyDataResult {
    pub data: i64,
}
