use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct VideoClient {
    client: BiliClient,
}

impl VideoClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取视频详细信息 (web端)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/view`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    pub async fn info(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<VideoInfo>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push_val("aid", a);
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        self.client
            .get("https://api.bilibili.com/x/web-interface/view", params)
            .await
    }

    /// 获取视频流地址 (web端, Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/player/wbi/playurl`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    /// - `cid`: 视频 cid
    /// - `qn`: 视频清晰度 (默认80=1080P, DASH格式时无效)
    ///   - 16=360P, 32=480P, 64=720P, 74=720P60, 80=1080P, 112=1080P+, 116=1080P60, 120=4K, 125=HDR, 126=杜比视界, 127=8K
    /// - `fnval`: 视频流格式标识 (默认16=DASH)
    ///   - 1=MP4, 16=DASH, 64=HDR, 128=4K, 256=杜比音频, 512=杜比视界, 1024=8K, 2048=AV1, 4048=所有DASH
    /// - `fourk`: 是否允许4K (0=否, 1=是)
    pub async fn playurl(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
        cid: i64,
        qn: Option<i64>,
        fnval: Option<i64>,
        fourk: Option<i64>,
    ) -> Result<BiliResponse<VideoUrlResult>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push_val("avid", a);
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        params = params.push_val("cid", cid);
        params = params.push_val("qn", qn.unwrap_or(80));
        params = params.push_val("fnval", fnval.unwrap_or(16));
        params = params.push("fnver", "0");
        if let Some(f) = fourk {
            params = params.push_val("fourk", f);
        }

        self.client
            .wbi_get("https://api.bilibili.com/x/player/wbi/playurl", params)
            .await
    }

    /// 获取当前热门视频列表
    ///
    /// `GET https://api.bilibili.com/x/web-interface/popular`
    ///
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    pub async fn popular(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<VideoHotResult>, BiliError> {
        let params = Params::new()
            .push_val("pn", pn.unwrap_or(1))
            .push_val("ps", ps.unwrap_or(20));
        self.client
            .get("https://api.bilibili.com/x/web-interface/popular", params)
            .await
    }

    /// 获取相关推荐视频
    ///
    /// `GET https://api.bilibili.com/x/web-interface/archive/related`
    ///
    /// - `aid`: 稿件 avid (与 bvid 任选一个)
    /// - `bvid`: 稿件 bvid (与 aid 任选一个)
    pub async fn related(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<VideoRecommendResult>, BiliError> {
        let mut params = Params::new();
        if let Some(a) = aid {
            params = params.push_val("aid", a);
        }
        if let Some(b) = bvid {
            params = params.push("bvid", b);
        }
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/archive/related",
                params,
            )
            .await
    }

    /// 获取首页推荐视频
    ///
    /// `GET https://api.bilibili.com/x/web-interface/index/top/feed/rcmd`
    ///
    /// - `fresh_type`: 刷新类型 (默认3)
    /// - `ps`: 数量 (默认20)
    pub async fn rcmd(&self, ps: Option<i64>) -> Result<BiliResponse<VideoRcmdResult>, BiliError> {
        let params = Params::new()
            .push("fresh_type", "3")
            .push("feed_version", "V8")
            .push("fresh_idx", "1")
            .push_val("ps", ps.unwrap_or(20));
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/index/top/feed/rcmd",
                params,
            )
            .await
    }

    /// 获取分区视频排行榜列表
    ///
    /// `GET https://api.bilibili.com/x/web-interface/ranking/v2`
    ///
    /// 该接口一次返回 100 条，且**不支持** `pn` / `ps` 参数，因此这里在客户端
    /// 对结果做本地分页，保持 CLI 的 `--pn` / `--ps` 语义。
    ///
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    /// - `rid`: 目标分区 tid (默认0=全站, 仅支持主分区)
    pub async fn hot(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
        rid: Option<&str>,
    ) -> Result<BiliResponse<VideoHotResult>, BiliError> {
        let params = Params::new().push("rid", rid.unwrap_or("0"));
        let resp = self
            .client
            .get::<VideoHotResult>(
                "https://api.bilibili.com/x/web-interface/ranking/v2",
                params,
            )
            .await?;
        Ok(paginate_hot(resp, pn.unwrap_or(1), ps.unwrap_or(20)))
    }

    /// Resolve a video's identifiers (`aid`, first-part `cid`, `pages`) from a `bvid`.
    ///
    /// This is the convenience entry point used by the CLI: every command that
    /// needs an `aid` or a `cid` can start from a `bvid`.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_ids(&self, bvid: &str) -> Result<VideoIds, BiliError> {
        let mut params = Params::new().push("bvid", bvid);
        self.fetch_ids(&mut params).await
    }

    /// Resolve a video's `(aid, cid)` from a `bvid`.
    ///
    /// `cid` is the first part's cid; use [`Self::resolve_ids`] to reach other
    /// parts of a multi-part video.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_bvid(&self, bvid: &str) -> Result<(i64, i64), BiliError> {
        let ids = self.resolve_ids(bvid).await?;
        Ok((ids.aid, ids.cid))
    }

    /// Resolve a video's `cid` from either an `aid` or a `bvid`.
    ///
    /// # Errors
    ///
    /// Propagates request errors, or returns [`BiliError::NullData`] when the
    /// endpoint returns no data.
    pub async fn resolve_cid(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<i64, BiliError> {
        let mut params = Params::new();
        if let Some(aid) = aid {
            params = params.push_val("aid", aid);
        }
        if let Some(bvid) = bvid {
            params = params.push("bvid", bvid);
        }
        Ok(self.fetch_ids(&mut params).await?.cid)
    }

    async fn fetch_ids(&self, params: &mut Params) -> Result<VideoIds, BiliError> {
        let endpoint = format!(
            "view?{}",
            params
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join("&")
        );
        let resp = self
            .client
            .get::<VideoInfo>(
                "https://api.bilibili.com/x/web-interface/view",
                std::mem::take(params),
            )
            .await?;
        let info = resp.data.ok_or(BiliError::NullData { endpoint })?;
        Ok(VideoIds {
            aid: info.aid,
            cid: info.cid,
            pages: info.pages,
        })
    }
}

/// Slice the 100-item ranking response according to `pn` / `ps`.
fn paginate_hot(
    mut resp: BiliResponse<VideoHotResult>,
    pn: i64,
    ps: i64,
) -> BiliResponse<VideoHotResult> {
    let pn = pn.max(1);
    let ps = ps.max(1);
    if let Some(data) = &mut resp.data {
        let start = usize::try_from(pn.saturating_sub(1).saturating_mul(ps)).unwrap_or(usize::MAX);
        let page_size = usize::try_from(ps).unwrap_or(usize::MAX);
        data.list = std::mem::take(&mut data.list)
            .into_iter()
            .skip(start)
            .take(page_size)
            .collect();
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hot_response(count: i64) -> BiliResponse<VideoHotResult> {
        BiliResponse {
            code: 0,
            message: String::new(),
            ttl: Some(1),
            data: Some(VideoHotResult {
                list: (0..count)
                    .map(|aid| VideoHotItem {
                        aid,
                        ..VideoHotItem::default()
                    })
                    .collect(),
                note: None,
            }),
        }
    }

    fn aids(resp: &BiliResponse<VideoHotResult>) -> Vec<i64> {
        resp.data
            .as_ref()
            .map(|d| d.list.iter().map(|v| v.aid).collect())
            .unwrap_or_default()
    }

    #[test]
    fn hot_paginates_locally() {
        let page = paginate_hot(hot_response(45), 2, 20);
        assert_eq!(aids(&page), (20..40).collect::<Vec<_>>());

        let last = paginate_hot(hot_response(45), 3, 20);
        assert_eq!(aids(&last), (40..45).collect::<Vec<_>>());

        let beyond = paginate_hot(hot_response(45), 4, 20);
        assert_eq!(aids(&beyond), Vec::<i64>::new());
    }

    #[test]
    fn hot_rejects_non_positive_page_params() {
        let page = paginate_hot(hot_response(5), 0, 0);
        assert_eq!(aids(&page), vec![0]);
    }

    #[test]
    fn video_ids_cid_of_page() {
        let ids = VideoIds {
            aid: 1,
            cid: 10,
            pages: vec![
                VideoPage {
                    cid: 10,
                    page: 1,
                    ..Default::default()
                },
                VideoPage {
                    cid: 20,
                    page: 2,
                    ..Default::default()
                },
            ],
        };
        assert_eq!(ids.cid_of_page(1), Some(10));
        assert_eq!(ids.cid_of_page(2), Some(20));
        assert_eq!(ids.cid_of_page(3), None);
    }
}

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
    #[serde(default)]
    pub dislike: i64,
    pub evaluation: String,
    pub vt: i64,
    #[serde(default)]
    pub vv: i64,
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
    /// 语言的默认信息
    pub lan: String,
    pub lan_doc: String,
    /// 字幕列表（接口字段名为 `subtitles`）
    #[serde(rename = "subtitles")]
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
#[allow(clippy::struct_excessive_bools)] // mirrors the upstream API shape
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
    pub premiere: Option<serde_json::Value>,
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

/// 视频标识，由 [`crate::BiliClient::resolve_video_ids`] 返回。
///
/// `cid` 为第一个分P 的 cid；多P 视频可用 [`VideoIds::cid_of_page`] 获取其它分P。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoIds {
    /// 稿件 avid
    pub aid: i64,
    /// 第一个分P 的 cid
    pub cid: i64,
    /// 全部分P
    pub pages: Vec<VideoPage>,
}

impl VideoIds {
    /// 获取指定分P（从 1 开始）的 cid。
    #[must_use]
    pub fn cid_of_page(&self, page: i64) -> Option<i64> {
        self.pages.iter().find(|p| p.page == page).map(|p| p.cid)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoHotResult {
    pub list: Vec<VideoHotItem>,
    pub note: Option<String>,
}

/// 热门视频条目
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoHotItem {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub pic: String,
    pub desc: String,
    pub duration: i64,
    pub owner: VideoOwner,
    pub stat: VideoStat,
    pub pubdate: i64,
    pub tname: String,
    pub r#type: i64,
    pub goto: String,
    pub uri: String,
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
    pub desc: String,
    /// 实际接口返回秒数（数字），文档旧示例给的是字符串，故保留原始 JSON。
    pub duration: serde_json::Value,
    pub pubdate: i64,
    pub tname: String,
    pub owner: Option<VideoOwner>,
    pub stat: Option<VideoStat>,
    pub r#type: Option<i64>,
    // 以下字段在旧文档/部分响应中存在，保留兼容。
    pub url: Option<String>,
    pub author: Option<String>,
    pub mid: Option<i64>,
    pub play: Option<i64>,
    pub danmaku: Option<i64>,
    pub is_live_playback: Option<bool>,
    pub is_follow: Option<bool>,
    pub is_upower_exclusive: Option<bool>,
    pub cnt_info: Option<VideoStat>,
}

/// `x/web-interface/archive/related` 的 `data` 直接就是推荐视频数组。
pub type VideoRecommendResult = Vec<VideoRecommendItem>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoUrlResult {
    #[serde(default)]
    pub v_voucher: String,
    #[serde(default)]
    pub from: String,
    #[serde(default)]
    pub result: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub quality: i64,
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub timelength: i64,
    #[serde(default)]
    pub accept_format: String,
    #[serde(default)]
    pub accept_description: Vec<String>,
    #[serde(default)]
    pub accept_quality: Vec<i64>,
    #[serde(default)]
    pub video_codecid: i64,
    #[serde(default)]
    pub seek_param: String,
    #[serde(default)]
    pub seek_type: String,
    pub durl: Vec<VideoUrlItem>,
    #[serde(default)]
    pub dash: Option<DashStream>,
    #[serde(default)]
    pub support_formats: Vec<SupportFormat>,
    #[serde(default)]
    pub high_format: Option<serde_json::Value>,
    #[serde(default)]
    pub last_play_time: i64,
    #[serde(default)]
    pub last_play_cid: i64,
    #[serde(default)]
    pub cur_language: String,
}

/// 支持的播放格式
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SupportFormat {
    pub quality: i64,
    pub format: String,
    pub new_description: String,
    pub display_desc: String,
    pub superscript: String,
    pub codecs: Vec<String>,
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

/// 首页推荐视频列表
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoRcmdResult {
    pub item: Vec<VideoRcmdItem>,
}

/// 首页推荐视频条目
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoRcmdItem {
    pub id: i64,
    pub bvid: String,
    pub title: String,
    pub pic: String,
    pub desc: String,
    pub duration: i64,
    pub owner: Option<VideoOwner>,
    pub stat: Option<VideoStat>,
    pub goto: String,
    pub uri: String,
    pub r#type: i64,
    pub jump: Option<VideoRcmdJump>,
}

/// 推荐视频跳转信息
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VideoRcmdJump {
    pub goto: String,
    pub uri: String,
}
