use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserClient {
    client: BiliClient,
}

impl UserClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取用户空间详细信息 (Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/space/wbi/acc/info`
    ///
    /// - `mid`: 目标用户 mid
    pub async fn info(&self, mid: i64) -> Result<BiliResponse<UserInfo>, BiliError> {
        let params = Params::new().push_val("mid", mid);
        self.client
            .wbi_get("https://api.bilibili.com/x/space/wbi/acc/info", params)
            .await
    }

    /// 获取当前登录用户自己的信息
    ///
    /// `GET https://api.bilibili.com/x/web-interface/nav`
    pub async fn mine(&self) -> Result<serde_json::Value, BiliError> {
        self.client
            .get_raw(
                "https://api.bilibili.com/x/web-interface/nav",
                Params::new(),
            )
            .await
    }

    /// 获取用户投稿视频列表 (Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/space/wbi/arc/search`
    ///
    /// - `mid`: 目标用户 mid
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认30)
    pub async fn videos(
        &self,
        mid: i64,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<UserVideoListResult>, BiliError> {
        let params = Params::new()
            .push_val("mid", mid)
            .push_val("pn", pn.unwrap_or(1))
            .push_val("ps", ps.unwrap_or(30));
        self.client
            .wbi_get("https://api.bilibili.com/x/space/wbi/arc/search", params)
            .await
    }

    /// 获取用户空间统计信息
    ///
    /// `GET https://api.bilibili.com/x/space/upstat`
    ///
    /// - `mid`: 目标用户 mid
    pub async fn stat(&self, mid: i64) -> Result<BiliResponse<UserStat>, BiliError> {
        let params = Params::new().push_val("mid", mid);
        self.client
            .get("https://api.bilibili.com/x/space/upstat", params)
            .await
    }

    /// 获取当前登录用户的 UID
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::CookieBuild`] if the store lock is poisoned, or
    /// [`BiliError::Parse`] if `DedeUserID` is missing or not an integer.
    pub fn get_current_uid(&self) -> Result<i64, BiliError> {
        let uid = self
            .client
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .get("DedeUserID")
            .map(String::from);
        uid.and_then(|s| s.parse().ok())
            .ok_or(BiliError::Parse("Cannot get current user UID".into()))
    }

    /// 关注用户
    ///
    /// `POST https://api.bilibili.com/x/relation/modify`
    ///
    /// - `fid`: 目标用户 mid
    pub async fn follow(&self, fid: i64) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;
        let form = FormBuilder::new()
            .push_val("fid", fid)
            .push("act", "1")
            .csrf(csrf)
            .build();
        self.client
            .post("https://api.bilibili.com/x/relation/modify", &form)
            .await
    }

    /// 取消关注用户
    ///
    /// `POST https://api.bilibili.com/x/relation/modify`
    ///
    /// - `fid`: 目标用户 mid
    pub async fn unfollow(&self, fid: i64) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;
        let form = FormBuilder::new()
            .push_val("fid", fid)
            .push("act", "2")
            .csrf(csrf)
            .build();
        self.client
            .post("https://api.bilibili.com/x/relation/modify", &form)
            .await
    }

    /// 移除粉丝
    ///
    /// `POST https://api.bilibili.com/x/relation/modify`
    ///
    /// - `fid`: 目标用户 mid
    pub async fn remove_follower(
        &self,
        fid: i64,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;
        let form = FormBuilder::new()
            .push_val("fid", fid)
            .push("act", "7")
            .csrf(csrf)
            .build();
        self.client
            .post("https://api.bilibili.com/x/relation/modify", &form)
            .await
    }

    /// 批量关注用户
    ///
    /// `POST https://api.bilibili.com/x/relation/batch/modify`
    ///
    /// - `fids`: 目标用户 mid 列表
    ///
    /// 返回失败的 fid 列表
    pub async fn batch_follow(
        &self,
        fids: &[i64],
    ) -> Result<BiliResponse<BatchModifyResult>, BiliError> {
        let csrf = self.client.csrf()?;
        let fids_str = fids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let form = FormBuilder::new()
            .push("fids", fids_str)
            .push("act", "1")
            .csrf(csrf)
            .build();
        self.client
            .post::<BatchModifyResult>("https://api.bilibili.com/x/relation/batch/modify", &form)
            .await
    }

    /// 批量拉黑用户
    ///
    /// `POST https://api.bilibili.com/x/relation/batch/modify`
    ///
    /// - `fids`: 目标用户 mid 列表
    ///
    /// 返回失败的 fid 列表
    pub async fn batch_block(
        &self,
        fids: &[i64],
    ) -> Result<BiliResponse<BatchModifyResult>, BiliError> {
        let csrf = self.client.csrf()?;
        let fids_str = fids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let form = FormBuilder::new()
            .push("fids", fids_str)
            .push("act", "5")
            .csrf(csrf)
            .build();
        self.client
            .post::<BatchModifyResult>("https://api.bilibili.com/x/relation/batch/modify", &form)
            .await
    }

    /// 查询用户关注明细
    ///
    /// `GET https://api.bilibili.com/x/relation/followings`
    ///
    /// 登录可看当前用户全部，其他用户仅可查看前 100 个
    ///
    /// - `vmid`: 目标用户 mid
    /// - `ps`: 每页项数 (默认50)
    /// - `pn`: 页码 (默认1)
    pub async fn followings(
        &self,
        vmid: i64,
        ps: Option<i64>,
        pn: Option<i64>,
    ) -> Result<BiliResponse<RelationFollowingResult>, BiliError> {
        let mut params = Params::new().push_val("vmid", vmid);
        if let Some(p) = ps {
            params = params.push_val("ps", p);
        }
        if let Some(p) = pn {
            params = params.push_val("pn", p);
        }
        self.client
            .get("https://api.bilibili.com/x/relation/followings", params)
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserOfficial {
    pub role: i64,
    pub title: String,
    pub desc: String,
    pub r#type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct UserAvatarIcon {
    pub icon_type: i64,
    pub icon_resource: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVip {
    pub r#type: i64,
    pub status: i64,
    pub due_date: i64,
    pub vip_pay_type: i64,
    pub theme_type: i64,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub label: UserVipLabel,
    pub avatar_subscript: i64,
    pub nickname_color: String,
    pub role: i64,
    pub avatar_subscript_url: String,
    pub tv_vip_status: i64,
    pub tv_vip_pay_type: i64,
    pub tv_due_date: i64,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub avatar_icon: UserAvatarIcon,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct UserNameplate {
    pub nid: i64,
    pub name: String,
    pub image: String,
    pub image_small: String,
    pub level: String,
    pub condition: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserFansMedalMedal {
    pub uid: i64,
    pub target_id: i64,
    pub medal_id: i64,
    pub level: i64,
    /// 粉丝勋章等级（部分响应中缺失）
    pub guard_level: Option<i64>,
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
#[serde(default)]
pub struct UserFansMedal {
    pub show: bool,
    pub wear: bool,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub medal: UserFansMedalMedal,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct UserLiveRoom {
    #[serde(rename = "roomStatus")]
    pub room_status: i64,
    #[serde(rename = "liveStatus")]
    pub live_status: i64,
    pub url: String,
    pub title: String,
    pub cover: String,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub watched_show: UserWatchedShow,
    pub roomid: i64,
    #[serde(rename = "roundStatus")]
    pub round_status: i64,
    pub broadcast_type: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserElecShowInfo {
    pub show: bool,
    pub state: i64,
    pub title: String,
    pub icon: String,
    pub jump_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserElec {
    pub show_info: Option<UserElecShowInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserContract {
    pub is_display: bool,
    pub is_follow_display: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserSchool {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserProfession {
    pub name: String,
    pub department: String,
    pub title: String,
    pub is_show: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserHonourInfo {
    pub mid: i64,
    pub colour: Option<String>,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub tags: Vec<serde_json::Value>,
    pub is_latest_100honour: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserSeries {
    pub user_upgrade_status: i64,
    pub show_upgrade_window: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
#[allow(clippy::struct_excessive_bools)] // mirrors the upstream API shape
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
    #[serde(default)]
    pub theme: serde_json::Value,
    pub sys_notice: Option<UserSysNotice>,
    pub live_room: Option<UserLiveRoom>,
    pub birthday: Option<String>,
    pub school: Option<UserSchool>,
    pub profession: Option<UserProfession>,
    pub tags: Option<Vec<String>>,
    pub series: Option<UserSeries>,
    pub is_senior_member: i64,
    pub mcn_info: Option<serde_json::Value>,
    pub gaia_res_type: i64,
    pub gaia_data: Option<serde_json::Value>,
    pub is_risk: bool,
    pub elec: Option<UserElec>,
    pub contract: Option<UserContract>,
    pub certificate_show: bool,
    pub name_render: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserStatItem {
    pub view: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserStat {
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub archive: UserStatItem,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub article: UserStatItem,
    pub likes: i64,
}

/// `x/space/wbi/arc/search` 返回的投稿视频条目。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVideoItem {
    pub comment: i64,
    pub typeid: i64,
    pub play: i64,
    pub pic: String,
    pub subtitle: String,
    pub description: String,
    /// 接口在部分响应中返回字符串 `"1"`，部分返回数字，故保留为原始 JSON。
    pub copyright: serde_json::Value,
    pub title: String,
    pub review: i64,
    pub author: String,
    pub mid: i64,
    /// 投稿时间（接口字段名是 `created`，不是 `pubdate`）。
    pub created: i64,
    /// 时长字符串（接口字段名是 `length`，如 `"22:11"`）。
    pub length: String,
    pub video_review: i64,
    pub aid: i64,
    pub bvid: String,
    pub hide_click: bool,
    pub is_pay: i64,
    pub is_union_video: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVideoTlistItem {
    pub tid: i64,
    pub count: i64,
    pub name: String,
}

/// `data.list`：接口返回三组数据而不是单个数组。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVideoList {
    /// 按分区 tid 分组的信息
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub tlist: std::collections::BTreeMap<String, UserVideoTlistItem>,
    /// 视频列表
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub vlist: Vec<UserVideoItem>,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub slist: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVideoPage {
    pub pn: i64,
    pub ps: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UserVideoListResult {
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub list: UserVideoList,
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub page: UserVideoPage,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BatchModifyResult {
    pub failed_fids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
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
#[serde(default)]
pub struct RelationFollowingResult {
    #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
    pub list: Vec<RelationFollowingItem>,
    pub re_version: i64,
    pub total: i64,
}
