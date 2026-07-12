use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentPage {
    pub num: i64,
    pub size: i64,
    pub count: i64,
    pub acount: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentConfig {
    pub showadmin: i64,
    pub showentry: i64,
    pub showfloor: i64,
    pub showtopic: i64,
    pub show_up_flag: bool,
    pub read_only: bool,
    pub show_del_log: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentLevelInfo {
    pub current_level: i64,
    pub current_min: i64,
    pub current_exp: i64,
    pub next_exp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMemberPendant {
    pub pid: i64,
    pub name: String,
    pub image: String,
    pub expire: i64,
    pub image_enhance: String,
    pub image_enhance_frame: String,
    pub n_pid: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMemberNameplate {
    pub nid: i64,
    pub name: String,
    pub image: String,
    pub image_small: String,
    pub level: String,
    pub condition: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMemberOfficialVerify {
    pub r#type: i64,
    pub desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMemberVipLabel {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
    pub use_img_label: Option<bool>,
    pub img_label_uri_hans: Option<String>,
    pub img_label_uri_hant: Option<String>,
    pub img_label_uri_hans_static: Option<String>,
    pub img_label_uri_hant_static: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMemberVip {
    pub vip_type: i64,
    pub vip_due_date: i64,
    pub due_remark: String,
    pub access_status: i64,
    pub vip_status: i64,
    pub vip_status_warn: String,
    pub theme_type: i64,
    pub label: CommentMemberVipLabel,
    pub avatar_subscript: i64,
    pub avatar_subscript_url: String,
    pub nickname_color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentMember {
    pub mid: String,
    pub uname: String,
    pub sex: String,
    pub sign: String,
    pub avatar: String,
    pub rank: String,
    pub display_rank: String,
    pub face_nft_new: i64,
    pub is_senior_member: i64,
    pub level_info: CommentLevelInfo,
    pub pendant: CommentMemberPendant,
    pub nameplate: CommentMemberNameplate,
    pub official_verify: CommentMemberOfficialVerify,
    pub vip: CommentMemberVip,
    pub fans_detail: Option<serde_json::Value>,
    pub following: i64,
    pub is_followed: i64,
    pub is_contractor: bool,
    pub contract_desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentContentEmoteMeta {
    pub size: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentContentEmote {
    pub id: i64,
    pub package_id: i64,
    pub state: i64,
    pub r#type: i64,
    pub attr: i64,
    pub text: String,
    pub url: String,
    pub meta: CommentContentEmoteMeta,
    pub mtime: i64,
    pub jump_title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentContent {
    pub message: String,
    pub plat: i64,
    pub device: String,
    pub members: Vec<serde_json::Value>,
    pub emote: Option<std::collections::HashMap<String, CommentContentEmote>>,
    pub jump_url: serde_json::Value,
    pub max_line: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentReplyFolder {
    pub has_folded: bool,
    pub is_folded: bool,
    pub rule: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentUpAction {
    pub like: bool,
    pub reply: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentReplyControl {
    pub time_desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentReply {
    pub rpid: i64,
    pub oid: i64,
    pub r#type: i64,
    pub mid: i64,
    pub root: i64,
    pub parent: i64,
    pub dialog: i64,
    pub count: i64,
    pub rcount: i64,
    pub state: i64,
    pub fansgrade: i64,
    pub attr: i64,
    pub ctime: i64,
    pub rpid_str: String,
    pub root_str: String,
    pub parent_str: String,
    pub like: i64,
    pub action: i64,
    pub member: CommentMember,
    pub content: CommentContent,
    pub replies: Vec<CommentReply>,
    pub assist: i64,
    pub folder: CommentReplyFolder,
    pub up_action: CommentUpAction,
    pub show_follow: bool,
    pub invisible: bool,
    pub reply_control: CommentReplyControl,
    pub mid_str: Option<String>,
    pub oid_str: Option<String>,
    pub dialog_str: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentUpper {
    pub mid: i64,
    pub top: Option<CommentReply>,
    pub vote: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentNotice {
    pub content: String,
    pub id: i64,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentFolder {
    pub has_folded: bool,
    pub is_folded: bool,
    pub rule: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentControl {
    pub input_disable: bool,
    pub root_input_text: String,
    pub child_input_text: String,
    pub bg_text: String,
    pub web_selection: bool,
    pub answer_guide_text: String,
    pub answer_guide_icon_url: String,
    pub answer_guide_ios_url: String,
    pub answer_guide_android_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentCursorPagination {
    pub next_offset: String,
    pub prev_offset: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentCursor {
    pub all_count: i64,
    pub is_begin: bool,
    pub prev: i64,
    pub next: i64,
    pub is_end: bool,
    pub mode: i64,
    pub support_mode: Vec<i64>,
    pub name: String,
    pub pagination_reply: CommentCursorPagination,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentList {
    pub page: CommentPage,
    pub config: CommentConfig,
    pub replies: Option<Vec<CommentReply>>,
    pub hots: Option<Vec<CommentReply>>,
    pub upper: CommentUpper,
    pub top: Option<serde_json::Value>,
    pub notice: Option<CommentNotice>,
    pub vote: i64,
    pub blacklist: i64,
    pub assist: i64,
    pub mode: i64,
    pub support_mode: Vec<i64>,
    pub folder: CommentFolder,
    pub show_bvid: bool,
    pub control: CommentControl,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentListWbi {
    pub cursor: CommentCursor,
    pub hots: Option<Vec<CommentReply>>,
    pub notice: Option<CommentNotice>,
    pub replies: Option<Vec<CommentReply>>,
    pub top: serde_json::Value,
    pub folder: CommentFolder,
    pub config: CommentConfig,
    pub upper: CommentUpper,
    pub show_bvid: bool,
    pub control: CommentControl,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CommentAddResult {
    pub success_action: i64,
    pub success_toast: String,
    pub need_captcha: bool,
    pub url: String,
    pub rpid: i64,
    pub rpid_str: String,
    pub dialog: i64,
    pub dialog_str: String,
    pub root: i64,
    pub root_str: String,
    pub parent: i64,
    pub parent_str: String,
}
