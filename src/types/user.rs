use serde::{Deserialize, Serialize};

use super::video::VideoRankPage;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserOfficial {
    pub role: i64,
    pub title: String,
    pub desc: String,
    pub r#type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserVipLabel {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
    pub use_img_label: bool,
    pub img_label_uri_hans: String,
    pub img_label_uri_hant: String,
    pub img_label_uri_hans_static: String,
    pub img_label_uri_hant_static: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserAvatarIcon {
    pub icon_type: i64,
    pub icon_resource: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserVip {
    pub r#type: i64,
    pub status: i64,
    pub due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
    pub label: UserVipLabel,
    pub avatar_subscript: i64,
    pub nickname_color: String,
    pub role: i64,
    pub avatar_subscript_url: String,
    pub tv_vip_status: i64,
    pub tv_vip_pay_type: i64,
    pub tv_due_date: i64,
    pub avatar_icon: UserAvatarIcon,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserPendant {
    pub pid: i64,
    pub name: String,
    pub image: String,
    pub expire: i64,
    pub image_enhance: String,
    pub image_enhance_frame: String,
    pub n_pid: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserNameplate {
    pub nid: i64,
    pub name: String,
    pub image: String,
    pub image_small: String,
    pub level: String,
    pub condition: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserFansMedalMedal {
    pub uid: i64,
    pub target_id: i64,
    pub medal_id: i64,
    pub level: i64,
    pub medal_name: String,
    pub medal_color: i64,
    pub intimacy: i64,
    pub next_intimacy: i64,
    pub day_limit: i64,
    pub today_feed: i64,
    pub medal_color_start: i64,
    pub medal_color_end: i64,
    pub medal_color_border: i64,
    pub is_lighted: i64,
    pub light_status: i64,
    pub wearing_status: i64,
    pub score: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserFansMedal {
    pub show: bool,
    pub wear: bool,
    pub medal: UserFansMedalMedal,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserWatchedShow {
    pub switch: bool,
    pub num: i64,
    pub text_small: String,
    pub text_large: String,
    pub icon: String,
    pub icon_location: String,
    pub icon_web: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserLiveRoom {
    pub room_status: i64,
    pub live_status: i64,
    pub url: String,
    pub title: String,
    pub cover: String,
    pub watched_show: UserWatchedShow,
    pub roomid: i64,
    pub round_status: i64,
    pub broadcast_type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserElecShowInfo {
    pub show: bool,
    pub state: i64,
    pub title: String,
    pub icon: String,
    pub jump_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserElec {
    pub show_info: Option<UserElecShowInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserContract {
    pub is_display: bool,
    pub is_follow_display: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSchool {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserProfession {
    pub name: String,
    pub department: String,
    pub title: String,
    pub is_show: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserHonourInfo {
    pub mid: i64,
    pub colour: Option<String>,
    pub tags: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSeries {
    pub user_upgrade_status: i64,
    pub show_upgrade_window: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserSysNotice {
    pub id: i64,
    pub content: String,
    pub url: String,
    pub notice_type: i64,
    pub icon: String,
    pub text_color: String,
    pub bg_color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserInfo {
    pub mid: i64,
    pub name: String,
    pub sex: String,
    pub face: String,
    pub face_nft: i64,
    pub face_nft_type: i64,
    pub sign: String,
    pub rank: i64,
    pub level: i64,
    pub jointime: i64,
    pub moral: i64,
    pub silence: i64,
    pub coins: f64,
    pub fans_badge: bool,
    pub fans_medal: Option<UserFansMedal>,
    pub official: Option<UserOfficial>,
    pub vip: Option<UserVip>,
    pub pendant: Option<UserPendant>,
    pub nameplate: Option<UserNameplate>,
    pub user_honour_info: Option<UserHonourInfo>,
    pub is_followed: bool,
    pub top_photo: String,
    pub theme: serde_json::Value,
    pub sys_notice: Option<UserSysNotice>,
    pub live_room: Option<UserLiveRoom>,
    pub birthday: Option<String>,
    pub school: Option<UserSchool>,
    pub profession: Option<UserProfession>,
    pub tags: Option<Vec<String>>,
    pub series: Option<UserSeries>,
    pub is_senior_member: i64,
    pub mcn_info: Option<()>,
    pub gaia_res_type: i64,
    pub gaia_data: Option<()>,
    pub is_risk: bool,
    pub elec: Option<UserElec>,
    pub contract: Option<UserContract>,
    pub certificate_show: bool,
    pub name_render: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserStat {
    pub mid: i64,
    pub archive_view: i64,
    pub article_view: i64,
    pub fans: i64,
    pub following: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserVideoItem {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub pic: String,
    pub play: i64,
    pub danmaku: i64,
    pub duration: String,
    pub pubdate: i64,
    pub mid: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct UserVideoListResult {
    pub list: Vec<UserVideoItem>,
    pub page: VideoRankPage,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct BatchModifyResult {
    pub failed_fids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct RelationFollowingItem {
    pub mid: i64,
    pub attribute: i64,
    pub mtime: i64,
    pub tag: Option<Vec<i64>>,
    pub special: i64,
    pub uname: String,
    pub face: String,
    pub sign: String,
    pub face_nft: Option<i64>,
    pub official_verify: Option<serde_json::Value>,
    pub vip: Option<serde_json::Value>,
    pub name_render: Option<serde_json::Value>,
    pub nft_icon: Option<String>,
    pub rec_reason: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct RelationFollowingResult {
    pub list: Vec<RelationFollowingItem>,
    pub re_version: i64,
    pub total: i64,
}
