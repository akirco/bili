use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavFolderUpper {
    pub mid: i64,
    pub name: String,
    pub face: String,
    pub followed: Option<bool>,
    pub vip_type: Option<i64>,
    pub vip_statue: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavFolderCntInfo {
    pub collect: i64,
    pub play: i64,
    pub thumb_up: i64,
    pub share: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavFolderInfo {
    /// 收藏夹 `media_id`（`fav-items --media-id` 使用此字段）
    pub id: i64,
    /// 收藏夹短 id（旧版标识，不用于 `media_id` 参数）
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
#[serde(default)]
pub struct FavFolderListResult {
    pub count: i64,
    pub list: Vec<FavFolderInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavResourceUpper {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavResourceCntInfo {
    pub collect: i64,
    pub play: i64,
    pub danmaku: i64,
    pub thumb_up: Option<i64>,
    pub share: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
    pub season: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavResourceListResult {
    pub info: FavFolderInfo,
    /// 空收藏夹或参数错误时接口可能返回 `null`
    #[serde(default, deserialize_with = "super::null_as_default")]
    pub medias: Vec<FavResource>,
    pub has_more: bool,
    pub ttl: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavResourceId {
    pub id: i64,
    pub r#type: i64,
    pub bv_id: String,
    pub bvid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavDealResult {
    pub prompt: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FavHasFavoriteResult {
    pub count: i64,
    pub favoured: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct EmptyDataResult {
    pub data: i64,
}
