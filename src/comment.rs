use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct CommentClient {
    client: BiliClient,
}

impl CommentClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取评论列表（旧接口，已废弃）
    ///
    /// `GET https://api.bilibili.com/x/v2/reply/main`
    ///
    /// B 站已改用 [`Self::list_wbi`]；该旧接口目前可能返回空数据或 412，
    /// 保留仅为兼容旧调用方。
    ///
    /// - `comment_type`: 评论区类型 (1=视频, 11=图文, 17=动态)
    /// - `oid`: 目标对象 id (视频为 avid)
    /// - `sort`: 排序方式 (0=按时间, 2=按点赞数, 3=按回复数, 默认0)
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    pub async fn list(
        &self,
        comment_type: i64,
        oid: i64,
        sort: Option<i64>,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<CommentList>, BiliError> {
        let params = Params::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("sort", sort.unwrap_or(0))
            .push_val("pn", pn.unwrap_or(1))
            .push_val("ps", ps.unwrap_or(20));
        self.client
            .get("https://api.bilibili.com/x/v2/reply/main", params)
            .await
    }

    /// 获取评论列表 (Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/v2/reply/wbi/main`
    ///
    /// - `comment_type`: 评论区类型 (1=视频, 11=图文, 17=动态)
    /// - `oid`: 目标对象 id (视频为 avid)
    /// - `mode`: 排序方式 (2=按点赞数, 3=按回复数)
    /// - `pagination_str`: 翻页参数, 从上次响应中获取
    pub async fn list_wbi(
        &self,
        comment_type: i64,
        oid: i64,
        mode: Option<i64>,
        pagination_str: Option<&str>,
    ) -> Result<BiliResponse<CommentListWbi>, BiliError> {
        let mut params = Params::new()
            .push_val("type", comment_type)
            .push_val("oid", oid);
        if let Some(m) = mode {
            params = params.push_val("mode", m);
        }
        if let Some(p) = pagination_str {
            params = params.push("pagination_str", p);
        }
        self.client
            .wbi_get("https://api.bilibili.com/x/v2/reply/wbi/main", params)
            .await
    }

    /// 发表评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/add`
    ///
    /// - `comment_type`: 评论区类型 (1=视频, 11=图文, 17=动态)
    /// - `oid`: 目标对象 id (视频为 avid)
    /// - `message`: 评论内容
    /// - `root`: 根评论 rpid (回复某评论时填写, 二级回复用)
    /// - `parent`: 父评论 rpid (回复某评论时填写, 一级回复用)
    /// - `plat`: 平台 (1=web)
    pub async fn add(
        &self,
        comment_type: i64,
        oid: i64,
        message: &str,
        root: Option<i64>,
        parent: Option<i64>,
        plat: Option<i64>,
    ) -> Result<BiliResponse<CommentAddResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push("message", message)
            .push_opt_val("root", root)
            .push_opt_val("parent", parent)
            .push_opt_val("plat", plat)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/add", &form)
            .await
    }

    /// 点赞评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/action`
    ///
    /// - `comment_type`: 评论区类型
    /// - `oid`: 目标对象 id
    /// - `rpid`: 评论 rpid
    /// - `action`: 1=点赞, 0=取消点赞
    pub async fn like(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("rpid", rpid)
            .push_opt_val("action", action)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/action", &form)
            .await
    }

    /// 踩评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/hate`
    ///
    /// - `comment_type`: 评论区类型
    /// - `oid`: 目标对象 id
    /// - `rpid`: 评论 rpid
    /// - `action`: 1=踩, 0=取消踩
    pub async fn hate(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("rpid", rpid)
            .push_opt_val("action", action)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/hate", &form)
            .await
    }

    /// 删除评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/del`
    ///
    /// - `comment_type`: 评论区类型
    /// - `oid`: 目标对象 id
    /// - `rpid`: 评论 rpid
    pub async fn delete(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("rpid", rpid)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/del", &form)
            .await
    }

    /// 置顶评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/top`
    ///
    /// - `comment_type`: 评论区类型
    /// - `oid`: 目标对象 id
    /// - `rpid`: 评论 rpid
    /// - `action`: 1=置顶, 0=取消置顶
    pub async fn top(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("rpid", rpid)
            .push_opt_val("action", action)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/top", &form)
            .await
    }

    /// 举报评论
    ///
    /// `POST https://api.bilibili.com/x/v2/reply/report`
    ///
    /// - `comment_type`: 评论区类型
    /// - `oid`: 目标对象 id
    /// - `rpid`: 评论 rpid
    /// - `reason`: 举报原因 (1=违法违禁, 2=色情低俗, 3=赌博诈骗, 4=人身攻击, 5=侵犯隐私, 6=垃圾广告, 7=引战, 8=剧透, 9=恶意刷屏, 10=视频无关, 11=其他)
    /// - `content`: 举报备注 (reason=11时有效)
    pub async fn report(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        reason: i64,
        content: Option<&str>,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("type", comment_type)
            .push_val("oid", oid)
            .push_val("rpid", rpid)
            .push_val("reason", reason)
            .push_opt("content", content)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/report", &form)
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentPage {
    pub num: i64,
    pub size: i64,
    pub count: i64,
    pub acount: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct CommentLevelInfo {
    pub current_level: i64,
    pub current_min: i64,
    pub current_exp: i64,
    pub next_exp: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct CommentMemberNameplate {
    pub nid: i64,
    pub name: String,
    pub image: String,
    pub image_small: String,
    pub level: String,
    pub condition: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentMemberOfficialVerify {
    pub r#type: i64,
    pub desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct CommentMemberVip {
    #[serde(rename = "vipType")]
    pub vip_type: i64,
    #[serde(rename = "vipDueDate")]
    pub vip_due_date: i64,
    #[serde(rename = "dueRemark")]
    pub due_remark: String,
    #[serde(rename = "accessStatus")]
    pub access_status: i64,
    #[serde(rename = "vipStatus")]
    pub vip_status: i64,
    #[serde(rename = "vipStatusWarn")]
    pub vip_status_warn: String,
    #[serde(rename = "themeType")]
    pub theme_type: i64,
    pub label: CommentMemberVipLabel,
    pub avatar_subscript: i64,
    pub avatar_subscript_url: String,
    pub nickname_color: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentMember {
    pub mid: String,
    pub uname: String,
    pub sex: String,
    pub sign: String,
    pub avatar: String,
    pub rank: String,
    /// 接口字段名是 `DisplayRank`（大写 D）。
    #[serde(rename = "DisplayRank", alias = "display_rank")]
    pub display_rank: String,
    pub face_nft_new: i64,
    pub is_senior_member: i64,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub level_info: CommentLevelInfo,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub pendant: CommentMemberPendant,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub nameplate: CommentMemberNameplate,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub official_verify: CommentMemberOfficialVerify,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub vip: CommentMemberVip,
    pub fans_detail: Option<serde_json::Value>,
    pub following: i64,
    pub is_followed: i64,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub user_sailing: CommentUserSailing,
    pub is_contractor: bool,
    pub contract_desc: String,
}

/// 评论者装扮信息
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUserSailing {
    pub pendant: Option<CommentUserSailingItem>,
    pub cardbg: Option<CommentUserSailingCard>,
    pub cardbg_with_focus: Option<CommentUserSailingCard>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUserSailingItem {
    pub id: i64,
    pub name: String,
    pub image: String,
    pub jump_url: String,
    pub r#type: String,
    pub image_enhance: Option<String>,
    pub image_enhance_frame: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUserSailingCard {
    pub id: i64,
    pub name: String,
    pub image: String,
    pub jump_url: String,
    pub fan: Option<CommentUserSailingFan>,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUserSailingFan {
    pub is_fan: i64,
    pub number: i64,
    pub color: String,
    pub name: String,
    pub num_desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentContentEmoteMeta {
    pub size: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct CommentContent {
    pub message: String,
    pub plat: i64,
    pub device: String,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub members: Vec<CommentContentMember>,
    pub emote: Option<std::collections::HashMap<String, CommentContentEmote>>,
    pub jump_url: serde_json::Value,
    pub max_line: i64,
}

/// @某人的信息
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentContentMember {
    pub mid: String,
    pub uname: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentReplyFolder {
    pub has_folded: bool,
    pub is_folded: bool,
    pub rule: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUpAction {
    pub like: bool,
    pub reply: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentReplyControl {
    pub time_desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
    #[serde(default)]
    pub floor: i64,
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
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub replies: Vec<CommentReply>,
    pub assist: i64,
    pub folder: CommentReplyFolder,
    pub up_action: CommentUpAction,
    pub show_follow: bool,
    pub invisible: bool,
    #[serde(default)]
    pub card_label: Option<serde_json::Value>,
    pub reply_control: CommentReplyControl,
    pub mid_str: Option<String>,
    pub oid_str: Option<String>,
    pub dialog_str: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentUpper {
    pub mid: i64,
    pub top: Option<CommentReply>,
    #[serde(default)]
    pub vote: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentNotice {
    pub content: String,
    pub id: i64,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentFolder {
    pub has_folded: bool,
    pub is_folded: bool,
    pub rule: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentControl {
    pub input_disable: bool,
    pub root_input_text: String,
    pub child_input_text: String,
    #[serde(default)]
    pub giveup_input_text: String,
    pub bg_text: String,
    pub web_selection: bool,
    pub answer_guide_text: String,
    pub answer_guide_icon_url: String,
    pub answer_guide_ios_url: String,
    pub answer_guide_android_url: String,
    #[serde(default)]
    pub show_type: i64,
    #[serde(default)]
    pub show_text: String,
    #[serde(default)]
    pub disable_jump_emote: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentCursorPagination {
    pub next_offset: String,
    pub prev_offset: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct CommentCursor {
    pub all_count: i64,
    pub is_begin: bool,
    pub prev: i64,
    pub next: i64,
    pub is_end: bool,
    pub mode: i64,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub support_mode: Vec<i64>,
    pub name: String,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub pagination_reply: CommentCursorPagination,
    pub session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub support_mode: Vec<i64>,
    pub folder: CommentFolder,
    #[serde(default)]
    pub lottery_card: Option<serde_json::Value>,
    pub show_bvid: bool,
    pub control: CommentControl,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
    #[serde(default)]
    pub up_selection: serde_json::Value,
    #[serde(default)]
    pub cm: serde_json::Value,
    #[serde(default)]
    pub cm_info: serde_json::Value,
    #[serde(default)]
    pub effects: serde_json::Value,
    #[serde(default)]
    pub assist: i64,
    #[serde(default)]
    pub blacklist: i64,
    #[serde(default)]
    pub vote: i64,
    #[serde(default)]
    pub lottery: i64,
    #[serde(default)]
    pub note: i64,
    #[serde(default)]
    pub esports_grade_card: Option<serde_json::Value>,
    #[serde(default)]
    pub callbacks: Option<serde_json::Value>,
    #[serde(default)]
    pub context_feature: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
