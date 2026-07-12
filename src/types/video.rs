use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoDescV2 {
    pub raw_text: String,
    pub r#type: i64,
    pub biz_id: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoRights {
    pub bp: i64,
    pub elec: i64,
    pub download: i64,
    pub movie: i64,
    pub pay: i64,
    pub hd5: i64,
    pub no_reprint: i64,
    pub autoplay: i64,
    pub ugc_pay: i64,
    pub is_cooperation: i64,
    pub ugc_pay_preview: i64,
    pub no_background: i64,
    pub clean_mode: i64,
    pub is_stein_gate: i64,
    pub is_360: i64,
    pub no_share: i64,
    pub arc_pay: i64,
    pub free_watch: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoOwner {
    pub mid: i64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoStat {
    pub aid: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub favorite: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub evaluation: String,
    pub vt: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoArgueInfo {
    pub argue_link: String,
    pub argue_msg: String,
    pub argue_type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoDimension {
    pub width: i64,
    pub height: i64,
    pub rotate: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoPage {
    pub cid: i64,
    pub page: i64,
    pub from: String,
    pub part: String,
    pub duration: i64,
    pub vid: String,
    pub weblink: String,
    pub dimension: Option<VideoDimension>,
    pub ctime: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoSubtitleAuthor {
    pub mid: i64,
    pub name: String,
    pub sex: String,
    pub face: String,
    pub sign: String,
    pub rank: i64,
    pub birthday: i64,
    pub is_fake_account: i64,
    pub is_deleted: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoSubtitleList {
    pub id: i64,
    pub lan: String,
    pub lan_doc: String,
    pub is_lock: bool,
    #[serde(default)]
    pub author_mid: Option<i64>,
    pub subtitle_url: String,
    pub author: VideoSubtitleAuthor,
    pub r#type: Option<i64>,
    pub id_str: Option<String>,
    pub ai_type: Option<i64>,
    pub ai_status: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoSubtitle {
    pub allow_submit: bool,
    pub list: Vec<VideoSubtitleList>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoVipLabel {
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

#[serde(default)]
pub struct VideoVip {
    pub r#type: i64,
    pub status: i64,
    pub due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
    pub label: VideoVipLabel,
    pub avatar_subscript: i64,
    pub nickname_color: String,
    pub role: i64,
    pub avatar_subscript_url: String,
    pub tv_vip_status: i64,
    pub tv_vip_pay_type: i64,
    pub tv_due_date: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoOfficial {
    pub role: i64,
    pub title: String,
    pub desc: String,
    pub r#type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoStaff {
    pub mid: i64,
    pub title: String,
    pub name: String,
    pub face: String,
    pub vip: VideoVip,
    pub official: VideoOfficial,
    pub follower: i64,
    pub label_style: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoHonor {
    pub aid: i64,
    pub r#type: i64,
    pub desc: String,
    pub weekly_recommend_num: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoHonorReply {
    pub honor: Vec<VideoHonor>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoUserGarb {
    pub url_image_ani_cut: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: i64,
    pub videos: i64,
    pub tid: i64,
    pub tid_v2: i64,
    pub tname: String,
    pub tname_v2: String,
    pub copyright: i64,
    pub pic: String,
    pub title: String,
    pub pubdate: i64,
    pub ctime: i64,
    pub desc: String,
    pub desc_v2: Option<Vec<VideoDescV2>>,
    pub state: i64,
    pub duration: i64,
    pub forward: Option<i64>,
    pub mission_id: Option<i64>,
    pub redirect_url: Option<String>,
    pub rights: VideoRights,
    pub owner: VideoOwner,
    pub stat: VideoStat,
    pub argue_info: VideoArgueInfo,
    pub dynamic: String,
    pub cid: i64,
    pub dimension: VideoDimension,
    pub premiere: Option<()>,
    pub teenage_mode: i64,
    pub is_chargeable_season: bool,
    pub is_story: bool,
    pub is_upower_exclusive: bool,
    pub is_upower_play: bool,
    pub is_upower_preview: bool,
    pub no_cache: bool,
    pub pages: Vec<VideoPage>,
    pub subtitle: VideoSubtitle,
    pub staff: Option<Vec<VideoStaff>>,
    pub is_season_display: bool,
    pub user_garb: VideoUserGarb,
    pub honor_reply: VideoHonorReply,
    pub like_icon: String,
    pub need_jump_bv: bool,
    pub disable_show_up_info: bool,
    pub is_story_play: i64,
    pub is_view_self: bool,
    pub enable_vt: Option<i64>,
    pub vt_display: Option<String>,
    pub is_upower_exclusive_with_qa: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoHotResult {
    pub list: Vec<serde_json::Value>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoRankPage {
    pub num: i64,
    pub size: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoRecommendItem {
    pub aid: i64,
    pub bvid: String,
    pub cid: i64,
    pub title: String,
    pub pic: String,
    pub url: String,
    pub play: i64,
    pub danmaku: i64,
    pub duration: String,
    pub author: String,
    pub mid: i64,
    pub r#type: i64,
    pub tname: String,
    pub pubdate: i64,
    pub is_live_playback: bool,
    pub is_follow: bool,
    pub is_upower_exclusive: bool,
    pub cnt_info: Option<VideoStat>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoRecommendResult {
    pub list: Vec<VideoRecommendItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoUrlResult {
    pub durl: Vec<VideoUrlItem>,
    #[serde(default)]
    pub dash: Option<DashStream>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct DashStream {
    pub duration: i64,
    pub min_buffer_time: f64,
    pub video: Vec<DashVideoItem>,
    pub audio: Vec<DashAudioItem>,
    #[serde(default)]
    pub dolby: Option<DashDolby>,
    #[serde(default)]
    pub flac: Option<DashFlac>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct DashVideoItem {
    pub id: i64,
    pub base_url: String,
    #[serde(default)]
    pub backup_url: Vec<String>,
    pub bandwidth: i64,
    pub mime_type: String,
    pub codecs: String,
    pub width: i64,
    pub height: i64,
    pub frame_rate: String,
    pub codecid: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct DashAudioItem {
    pub id: i64,
    pub base_url: String,
    #[serde(default)]
    pub backup_url: Vec<String>,
    pub bandwidth: i64,
    pub mime_type: String,
    pub codecs: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct DashDolby {
    #[serde(default)]
    pub audio: Option<Vec<DashAudioItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct DashFlac {
    pub audio: Option<DashAudioItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

#[serde(default)]
pub struct VideoUrlItem {
    pub order: i64,
    pub length: i64,
    pub size: i64,
    pub ahead: String,
    pub vhead: String,
    pub url: String,
    pub backup_url: Vec<String>,
}
