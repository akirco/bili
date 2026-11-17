//! 直播相关接口：直播间搜索、直播流地址、直播分区数据。
//!
//! 只覆盖三块能力，接口都来自 web 端（`live.bilibili.com` / `api.live.bilibili.com`）：
//!
//! - [`LiveClient::search`] — 按关键字搜直播间
//! - [`LiveClient::media`] — 取某个直播间的播放流（FLV / HLS）
//! - [`LiveClient::categories`] — 直播分区列表（父分区 + 子分区）

use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const LIVE_API: &str = "https://api.live.bilibili.com";
const LIVE_WEB: &str = "https://live.bilibili.com";

#[derive(Clone)]
pub struct LiveClient {
    client: BiliClient,
    /// 分区 `(parent_area_id, area_id)` → `w_webid`。
    ///
    /// `access_id` 是按分区下发的 gaia 令牌（约一天有效），翻页时复用同一份，
    /// 免得每翻一页都重抓一次分区页 HTML。
    webid_cache: Arc<RwLock<HashMap<(i64, i64), String>>>,
}

impl LiveClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
            webid_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 搜索直播间 (Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/type`
    /// (`search_type=live_room`)
    ///
    /// 走的是综合搜索的分类接口而不是 `live.bilibili.com` 的 `searchType`：
    /// 两者的结果字段基本一致，但前者已经是本crate里做过签名的公开入口，
    /// 且不需要额外的 `qv_id` 之类的反爬参数。
    ///
    /// - `keyword`: 需要搜索的关键词
    /// - `page`: 页码
    /// - `page_size`: 每页项数 (默认20)
    pub async fn search(
        &self,
        keyword: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<BiliResponse<LiveSearchResult>, BiliError> {
        let mut params = Params::new()
            .push("search_type", "live_room")
            .push("keyword", keyword)
            // 综合搜索按相关度排序；直播间没有独立的排序开关
            .push("order", "totalrank");
        if let Some(p) = page {
            params = params.push_val("page", p);
        }
        if let Some(ps) = page_size {
            params = params.push_val("page_size", ps);
        }
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/wbi/search/type",
                params,
            )
            .await
    }

    /// 获取直播间的播放流 (Wbi签名)
    ///
    /// `GET https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo`
    ///
    /// 返回值是一棵「协议 → 格式 → 编码 → 地址」四层嵌套的结构：
    ///
    /// - `protocol_name`：`http_stream`（HTTP-FLV）或 `http_hls`（HLS）
    /// - `format_name`：`flv` / `ts` / `fmp4`
    /// - `codec_name`：`avc` / `hevc`
    ///
    /// 其中 `ts` 和 `fmp4` 都是 HLS 播放列表（`base_url` 返回的是 m3u8），
    /// 区别只在切片容器；`flv` 才是可直接拉流的 FLV。
    ///
    /// 用 [`LivePlayInfo::streams`] 把四层结构拍平成一个列表更方便使用。
    ///
    /// - `room_id`: 直播间号（短号 `short_id` 也能用，B站会自行跳转）
    /// - `qn`: 画质，0 表示由服务端决定；可用画质见 `g_qn_desc`
    pub async fn media(
        &self,
        room_id: i64,
        qn: Option<i64>,
    ) -> Result<BiliResponse<LivePlayInfo>, BiliError> {
        let params = Params::new()
            .push_val("room_id", room_id)
            // 0=FLV, 1=HLS，两个都要才拿得到完整列表
            .push("protocol", "0,1")
            .push("format", "0,1,2")
            .push("codec", "0,1")
            .push_val("qn", qn.unwrap_or(0))
            .push("platform", "web")
            .push_val("ptype", 8)
            .push_val("dolby", 5)
            .push_val("panorama", 1)
            .push("eotf", "0,1")
            .push("qrcode", "true")
            .push_val("req_reason", 0)
            // 直播接口的固定埋点值，缺了会 -352
            .push("web_location", "444.8");
        self.client
            .wbi_get(
                &format!("{LIVE_API}/xlive/web-room/v2/index/getRoomPlayInfo"),
                params,
            )
            .await
    }

    /// 直播分区列表
    ///
    /// `GET https://api.live.bilibili.com/xlive/web-interface/v1/index/getWebAreaList`
    ///
    /// `data.data` 是父分区数组，每个父分区带一个 `list` 装子分区。
    /// 用 [`LiveAreaList::flat`] 可以直接拿到「父分区 + 子分区」拍平后的一层列表。
    ///
    /// 注意父分区 id 是数字、子分区 id 是字符串（`"86"`），这是接口本身的
    /// 序列化不一致，所以两级的 `id` 字段类型不同。
    pub async fn categories(&self) -> Result<BiliResponse<LiveAreaList>, BiliError> {
        let params = Params::new().push_val("source_id", 2);
        self.client
            .get(
                &format!("{LIVE_API}/xlive/web-interface/v1/index/getWebAreaList"),
                params,
            )
            .await
    }

    /// 按分区取直播间列表 (Wbi签名)
    ///
    /// `GET https://api.live.bilibili.com/xlive/web-interface/v1/second/getList`
    ///
    /// 这是「直播分区页」背后的接口，能拿到某个分区**当前正在直播**的房间，
    /// 按人气排序，一页 40 条。
    ///
    /// # 必须传 `w_webid`
    ///
    /// 该接口没有 `w_webid` 会直接返回 `-352`。这个值不是常量，而是从分区页
    /// HTML（`live.bilibili.com/p/eden/area-tags`）的 `window._render_data_.access_id`
    /// 里取出来的 gaia 令牌，绑定 buvid 与 UA、有效期约一天。所以本方法会先抓
    /// 一次分区页拿令牌，再发签名请求；同一 [`LiveClient`] 内按分区缓存，
    /// 避免翻页时每页都抓一次 HTML。
    ///
    /// # 匿名可用性
    ///
    /// 实测匿名请求也会被拦（`-352`），带登录 Cookie 才能稳定拿到数据。
    /// 需要先用 [`BiliClient::login`](crate::BiliClient) 或 `--cookies` 登录。
    ///
    /// # 参数
    ///
    /// - `parent_area_id`: 父分区 id（见 [`LiveAreaParent::id`]）
    /// - `area_id`: 子分区 id；传 `0` 表示整个父分区
    /// - `page`: 页码，从 1 开始
    pub async fn areas(
        &self,
        parent_area_id: i64,
        area_id: i64,
        page: Option<i64>,
    ) -> Result<BiliResponse<LiveRoomList>, BiliError> {
        let page = page.unwrap_or(1);
        let webid = self.area_webid(parent_area_id, area_id).await?;

        let params = Params::new()
            .push("platform", "web")
            .push_val("parent_area_id", parent_area_id)
            .push_val("area_id", area_id)
            // 空串表示按默认（人气）排序；显式传 online 也可以
            .push("sort_type", "")
            .push_val("page", page)
            .push("vajra_business_key", "")
            .push("web_location", "444.253")
            .push("w_webid", webid);
        self.client
            .wbi_get(
                &format!("{LIVE_API}/xlive/web-interface/v1/second/getList"),
                params,
            )
            .await
    }

    /// 取分区页的 `access_id`（即 `second/getList` 的 `w_webid`）。
    ///
    /// 抓不到时返回空串，交给接口自己报 `-352`，这样错误信息仍是上游的原文。
    async fn area_webid(&self, parent_area_id: i64, area_id: i64) -> Result<String, BiliError> {
        let key = (parent_area_id, area_id);
        if let Some(cached) = self.webid_cache.read().await.get(&key) {
            return Ok(cached.clone());
        }

        let params = Params::new()
            .push_val("parentAreaId", parent_area_id)
            .push_val("areaId", area_id);
        let html = self
            .client
            .get_text(&format!("{LIVE_WEB}/p/eden/area-tags"), params)
            .await?;
        let webid = extract_access_id(&html).unwrap_or_default();
        self.webid_cache.write().await.insert(key, webid.clone());
        Ok(webid)
    }
}

/// 从 `"access_id":"<jwt>"` 里抠出令牌。
///
/// 不用正则：只需要找一次固定前缀，再读到下一个引号即可。
fn extract_access_id(html: &str) -> Option<String> {
    const NEEDLE: &str = "\"access_id\"";
    let start = html.find(NEEDLE)?;
    let rest = &html[start + NEEDLE.len()..];
    // 跳过 `:` 和空白
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let end = rest.find('"')?;
    let value = &rest[..end];
    (!value.is_empty()).then(|| value.to_string())
}

/// `second/getList` 的 `data`：某个分区当前正在直播的房间。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveRoomList {
    /// 该分区当前开播房间总数
    pub count: i64,
    /// 1 = 还有下一页
    pub has_more: i64,
    pub list: Vec<LiveAreaRoom>,
    pub banner: serde_json::Value,
    pub new_tags: serde_json::Value,
    pub vajra: serde_json::Value,
}

impl LiveRoomList {
    /// 是否还有下一页。
    #[must_use]
    pub fn has_next(&self) -> bool {
        self.has_more == 1 && !self.list.is_empty()
    }
}

/// 分区列表里的直播间。
///
/// 与搜索结果不同，这里**没有** `live_status` 字段——能进这个列表本身就代表
/// 正在直播。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveAreaRoom {
    pub roomid: i64,
    pub uid: i64,
    pub uname: String,
    /// 头像
    pub face: String,
    pub title: String,
    /// 封面
    pub cover: String,
    pub system_cover: String,
    pub user_cover: String,
    /// 当前人气值
    pub online: i64,
    /// 子分区 id（数值）
    pub area_id: i64,
    pub area_name: String,
    pub area_v2_id: i64,
    pub area_v2_name: String,
    pub area_v2_parent_id: i64,
    pub area_v2_parent_name: String,
    pub parent_id: i64,
    /// 实测该字段会返回 `null`，而不是缺省或空串
    #[serde(deserialize_with = "crate::serde_util::null_as_default")]
    pub parent_name: String,
    /// 开播/关播相关标记，含义由上游定义
    pub flag: i64,
    pub live_key: String,
    pub session_id: String,
    pub is_auto_play: i64,
    /// 看过的人数（展示用）
    pub watched_show: serde_json::Value,
    pub verify: serde_json::Value,
}

impl LiveAreaRoom {
    /// 直播间地址。
    #[must_use]
    pub fn url(&self) -> String {
        format!("{LIVE_WEB}/{}", self.roomid)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveSearchResult {
    pub seid: String,
    pub page: i64,
    pub pagesize: i64,
    #[serde(rename = "numResults")]
    pub num_results: i64,
    #[serde(rename = "numPages")]
    pub num_pages: i64,
    pub suggest_keyword: String,
    pub rqt_type: String,
    #[serde(rename = "cost_time")]
    pub cost_time: serde_json::Value,
    pub exp_list: serde_json::Value,
    #[serde(rename = "egg_hit")]
    pub egg_hit: i64,
    pub pageinfo: serde_json::Value,
    /// `search_type=live_room` 时这里是直播间数组。
    ///
    /// 个别情况下会退化成 `{"live_room": [...]}` 这种对象形式，用
    /// [`Self::rooms`] 读取可以兼容两种形状。
    pub result: serde_json::Value,
    pub show_column: i64,
}

impl LiveSearchResult {
    /// 取出直播间列表，兼容 `result` 为数组或 `{live_room: [...]}` 两种情况。
    #[must_use]
    pub fn rooms(&self) -> Vec<LiveRoomBrief> {
        let array = if self.result.is_array() {
            self.result.clone()
        } else {
            self.result
                .get("live_room")
                .cloned()
                .unwrap_or(serde_json::Value::Array(Vec::new()))
        };
        serde_json::from_value(array).unwrap_or_default()
    }
}

/// 直播间搜索结果项。
///
/// 字段来自真实响应；接口偶尔省略部分键，所以整体 `#[serde(default)]`。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveRoomBrief {
    /// 搜索结果类型，固定为 `live_room`
    #[serde(rename = "type")]
    pub kind: String,
    /// 直播间号，等价于 `roomid`
    pub id: i64,
    pub roomid: i64,
    pub short_id: i64,
    /// 主播 uid
    pub uid: i64,
    pub uname: String,
    pub uface: String,
    pub title: String,
    /// `//i0.hdslb.com/...` 协议相对地址，使用时需补 `https:`
    pub cover: String,
    pub user_cover: String,
    /// 子分区 id（对应 [`LiveAreaChild::id`] 的数值形式）
    pub area: i64,
    /// 子分区名，如「其他单机」。
    ///
    /// 命中关键字时 B站会用 `<em class="keyword">…</em>` 包住匹配片段，
    /// 需要纯文本时用 [`Self::cate_name_text`]。
    pub cate_name: String,
    /// 1 = 直播中，0 = 未开播，2 = 轮播
    pub live_status: i64,
    /// 开播时间，形如 `2026-11-17 17:09:21`
    pub live_time: String,
    /// 当前在线人气值
    pub online: i64,
    pub attentions: i64,
    /// 主播打的标签，逗号分隔
    pub tags: String,
    pub is_fold: bool,
    pub is_live_room_inline: i64,
    pub live_room_inline: serde_json::Value,
    pub rank_index: i64,
    pub rank_offset: i64,
    pub hit_columns: Vec<String>,
}

impl LiveRoomBrief {
    /// 去掉 `cate_name` 里的 `<em>` 高亮标签。
    #[must_use]
    pub fn cate_name_text(&self) -> String {
        strip_em_tags(&self.cate_name)
    }

    /// 去掉 `title` 里的 `<em>` 高亮标签。
    #[must_use]
    pub fn title_text(&self) -> String {
        strip_em_tags(&self.title)
    }
}

/// 去掉搜索结果里的 `<em class="keyword">…</em>`，只留文字。
fn strip_em_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        if let Some(end) = rest[start..].find('>') {
            rest = &rest[start + end + 1..];
        } else {
            // 没有闭合的 `>`，把剩余部分原样保留
            rest = &rest[start..];
            break;
        }
    }
    out.push_str(rest);
    out
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayInfo {
    pub room_id: i64,
    pub short_id: i64,
    pub uid: i64,
    /// 1 = 直播中，0 = 未开播，2 = 轮播
    pub live_status: i64,
    pub live_time: i64,
    /// 未开播时整个字段为 `null`
    pub playurl_info: Option<LivePlayUrlInfo>,
    /// 是否开启「付费直播」
    pub is_hidden: bool,
    pub encrypted: bool,
    pub pwd_verified: bool,
    pub all_special_types: Vec<serde_json::Value>,
}

impl LivePlayInfo {
    /// 把「协议 → 格式 → 编码 → 地址」四层结构拍平成一个列表。
    ///
    /// 未开播（`playurl_info` 为 `null`）时返回空列表。
    #[must_use]
    pub fn streams(&self) -> Vec<LiveStream> {
        let Some(info) = &self.playurl_info else {
            return Vec::new();
        };
        let Some(playurl) = &info.playurl else {
            return Vec::new();
        };

        let mut out = Vec::new();
        for stream in &playurl.stream {
            for format in &stream.format {
                for codec in &format.codec {
                    for url in &codec.url_info {
                        out.push(LiveStream {
                            protocol: stream.protocol_name.clone(),
                            format: format.format_name.clone(),
                            codec: codec.codec_name.clone(),
                            quality: codec.current_qn,
                            // host 以 `/` 结尾，base_url 以 `/` 开头会拼出双斜杠，
                            // 这里统一裁掉一侧
                            url: format!(
                                "{}{}{}",
                                url.host.trim_end_matches('/'),
                                codec.base_url,
                                url.extra
                            ),
                            host: url.host.clone(),
                            extra: url.extra.clone(),
                            referer: info
                                .playurl
                                .as_ref()
                                .map(|p| p.referer.clone())
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }
        out
    }

    /// 优先取 HTTP-FLV 地址（[`Self::streams`] 里第一个 `flv`），没有则退化成第一个地址。
    #[must_use]
    pub fn best_flv(&self) -> Option<LiveStream> {
        self.streams().into_iter().find(|s| s.format == "flv")
    }

    /// 优先取 HLS（m3u8）地址，没有则返回 `None`。
    #[must_use]
    pub fn best_hls(&self) -> Option<LiveStream> {
        self.streams()
            .into_iter()
            .find(|s| s.protocol == "http_hls")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayUrlInfo {
    pub conf_json: String,
    pub playurl: Option<LivePlayUrl>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayUrl {
    /// 拉流时必须带的 `Referer`
    pub referer: String,
    pub cid: i64,
    /// 当前返回的清晰度（数值形式）
    pub qn: i64,
    pub stream: Vec<LivePlayStream>,
    /// 各清晰度的名称，如 `30000` → `杜比`
    pub g_qn_desc: Vec<LiveQnDesc>,
    /// 若因未登录/未开播等原因降级播放，这里说明原因
    pub p2p_data: serde_json::Value,
    pub dolby_qn: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveQnDesc {
    pub qn: i64,
    pub desc: String,
    pub hdr_desc: String,
    pub attr_desc: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayStream {
    /// `http_stream`（FLV）或 `http_hls`（HLS）
    pub protocol_name: String,
    pub format: Vec<LivePlayFormat>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayFormat {
    /// `flv` / `ts` / `fmp4`
    pub format_name: String,
    pub codec: Vec<LivePlayCodec>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayCodec {
    /// `avc`（H.264）或 `hevc`（H.265）
    pub codec_name: String,
    /// 该编码提供哪些画质
    pub accept_qn: Vec<i64>,
    /// 当前返回的画质
    pub current_qn: i64,
    pub base_url: String,
    pub url_info: Vec<LivePlayUrlEntry>,
    /// 相对地址形式（部分场景下 `base_url` 为空）
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LivePlayUrlEntry {
    /// 形如 `https://cn-xxx.bilivideo.com/`，以 `/` 结尾
    pub host: String,
    /// CDN 附加参数，形如 `?expires=...&sign=...`
    pub extra: String,
    /// 只有 P2P 场景才非空
    pub stream_ttl: i64,
}

/// [`LivePlayInfo::streams`] 拍平后的单条流地址。
#[derive(Debug, Clone, Serialize, Default)]
pub struct LiveStream {
    pub protocol: String,
    pub format: String,
    pub codec: String,
    pub quality: i64,
    /// 可直接请求的完整地址
    pub url: String,
    pub host: String,
    pub extra: String,
    /// 请求 `url` 时需要携带的 `Referer`
    pub referer: String,
}

impl LiveStream {
    /// 是否是 HTTP-FLV 流（可用 ffmpeg 直接录制）。
    #[must_use]
    pub fn is_flv(&self) -> bool {
        self.format == "flv"
    }

    /// 是否是 HLS 流（地址是 m3u8 播放列表）。
    ///
    /// `fmp4` 格式也是 HLS，只是切片容器不同，因此这里按协议名判断而不是格式名。
    #[must_use]
    pub fn is_hls(&self) -> bool {
        self.protocol == "http_hls"
    }
}

/// `getWebAreaList` 的 `data`。
///
/// 真正的分区数组在 `data.data` 里（`expid` 是 A/B 实验号）。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveAreaList {
    pub expid: i64,
    pub data: Vec<LiveAreaParent>,
}

impl LiveAreaList {
    /// 拍平成一层：先父分区本身，再紧跟它的所有子分区。
    ///
    /// 返回 `(parent, Option<child>)`，`None` 表示这条是父分区自己。
    #[must_use]
    pub fn flat(&self) -> Vec<(&LiveAreaParent, Option<&LiveAreaChild>)> {
        let mut out = Vec::new();
        for parent in &self.data {
            out.push((parent, None));
            for child in &parent.list {
                out.push((parent, Some(child)));
            }
        }
        out
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveAreaParent {
    /// 父分区 id，**数字**
    pub id: i64,
    pub name: String,
    pub list: Vec<LiveAreaChild>,
}

/// 子分区。
///
/// 接口里子分区的 `id` / `parent_id` 都是**字符串**（`"86"` / `"2"`），
/// 与父分区 id 的数字类型不一致。`pk_status` / `lock_status` / `act_id`
/// 也是字符串（多数为空串），`area_type` 反而是数字。
/// 这里保留原始类型，需要数值时用 [`LiveAreaChild::id_num`]。
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LiveAreaChild {
    pub id: String,
    pub parent_id: String,
    pub parent_name: String,
    pub name: String,
    pub pinyin: String,
    /// 旧版分区 id，部分老接口还在用
    pub old_area_id: String,
    /// 活动 id，空串表示没有
    pub act_id: String,
    pub pk_status: String,
    pub hot_status: i64,
    pub lock_status: String,
    /// 分区图标
    pub pic: String,
    /// 行业分类 id，如 `2`（网游）/ `1`（娱乐）
    pub area_type: i64,
}

impl LiveAreaChild {
    /// 把字符串 id 解析成数字（`"86"` → `86`），解析失败返回 `None`。
    #[must_use]
    pub fn id_num(&self) -> Option<i64> {
        self.id.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms_accepts_array_and_object_shapes() {
        let array = LiveSearchResult {
            result: serde_json::json!([
                {"roomid": 1, "uname": "a", "title": "t"},
                {"roomid": 2, "uname": "b", "title": "u"}
            ]),
            ..Default::default()
        };
        assert_eq!(array.rooms().len(), 2);
        assert_eq!(array.rooms()[0].roomid, 1);

        let object = LiveSearchResult {
            result: serde_json::json!({"live_room": [{"roomid": 3}]}),
            ..Default::default()
        };
        assert_eq!(object.rooms()[0].roomid, 3);

        let empty = LiveSearchResult::default();
        assert!(empty.rooms().is_empty());
    }

    #[test]
    fn stream_url_join_avoids_double_slash() {
        let info = LivePlayInfo {
            playurl_info: Some(LivePlayUrlInfo {
                playurl: Some(LivePlayUrl {
                    referer: "https://live.bilibili.com/".to_string(),
                    stream: vec![LivePlayStream {
                        protocol_name: "http_stream".to_string(),
                        format: vec![LivePlayFormat {
                            format_name: "flv".to_string(),
                            codec: vec![LivePlayCodec {
                                codec_name: "avc".to_string(),
                                current_qn: 10000,
                                base_url: "/live-bvc/123/live_456.flv".to_string(),
                                url_info: vec![LivePlayUrlEntry {
                                    host: "https://cn-hbxy-cu-01.bilivideo.com/".to_string(),
                                    extra: "?expires=1&sign=abc".to_string(),
                                    stream_ttl: 0,
                                }],
                                ..Default::default()
                            }],
                        }],
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let streams = info.streams();
        assert_eq!(streams.len(), 1);
        assert_eq!(
            streams[0].url,
            "https://cn-hbxy-cu-01.bilivideo.com/live-bvc/123/live_456.flv?expires=1&sign=abc"
        );
        assert!(streams[0].is_flv());
        assert!(!streams[0].is_hls());
        assert_eq!(streams[0].referer, "https://live.bilibili.com/");
        assert!(info.best_flv().is_some());
    }

    #[test]
    fn fmp4_counts_as_hls_not_flv() {
        let info = LivePlayInfo {
            playurl_info: Some(LivePlayUrlInfo {
                playurl: Some(LivePlayUrl {
                    stream: vec![LivePlayStream {
                        protocol_name: "http_hls".to_string(),
                        format: vec![LivePlayFormat {
                            format_name: "fmp4".to_string(),
                            codec: vec![LivePlayCodec {
                                current_qn: 15000,
                                base_url: "/live-bvc/abc/index.m3u8".to_string(),
                                url_info: vec![LivePlayUrlEntry {
                                    host: "https://cn.bilivideo.com".to_string(),
                                    ..Default::default()
                                }],
                                ..Default::default()
                            }],
                        }],
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        let streams = info.streams();
        assert_eq!(
            streams[0].url,
            "https://cn.bilivideo.com/live-bvc/abc/index.m3u8"
        );
        assert!(streams[0].is_hls());
        assert!(!streams[0].is_flv());
        assert!(info.best_hls().is_some());
        assert!(info.best_flv().is_none());
    }

    #[test]
    fn offline_room_yields_no_streams() {
        let info = LivePlayInfo {
            live_status: 0,
            playurl_info: None,
            ..Default::default()
        };
        assert!(info.streams().is_empty());
        assert!(info.best_flv().is_none());
        assert!(info.best_hls().is_none());
    }

    #[test]
    fn area_list_deserializes_nested_ids() {
        // 真实形状：父 id 是数字，子 id / parent_id / pk_status 是字符串，
        // area_type 是数字，父分区没有 weight
        let raw = serde_json::json!({
            "expid": 13,
            "data": [
                {
                    "id": 2,
                    "name": "网游",
                    "list": [
                        {
                            "id": "86",
                            "parent_id": "2",
                            "parent_name": "网游",
                            "name": "英雄联盟",
                            "pinyin": "",
                            "old_area_id": "",
                            "act_id": "",
                            "hot_status": 0,
                            "pk_status": "",
                            "lock_status": "",
                            "pic": "http://i0.hdslb.com/bfs/vc/x.png",
                            "area_type": 2
                        }
                    ]
                }
            ]
        });
        let list: LiveAreaList = serde_json::from_value(raw).unwrap();
        assert_eq!(list.expid, 13);
        assert_eq!(list.data[0].id, 2);
        assert_eq!(list.data[0].list[0].id, "86");
        assert_eq!(list.data[0].list[0].id_num(), Some(86));
        assert_eq!(list.data[0].list[0].area_type, 2);

        let flat = list.flat();
        assert_eq!(flat.len(), 2);
        assert!(flat[0].1.is_none());
        assert_eq!(flat[1].1.unwrap().name, "英雄联盟");
    }

    #[test]
    fn area_child_id_num_tolerates_garbage() {
        let child = LiveAreaChild {
            id: "not-a-number".to_string(),
            ..Default::default()
        };
        assert_eq!(child.id_num(), None);
    }

    #[test]
    fn strips_em_tags_from_search_highlight() {
        let room = LiveRoomBrief {
            cate_name: r#"<em class="keyword">原神</em>"#.to_string(),
            title: r#"深渊<em class="keyword">原神</em>刷新"#.to_string(),
            ..Default::default()
        };
        assert_eq!(room.cate_name_text(), "原神");
        assert_eq!(room.title_text(), "深渊原神刷新");
    }

    #[test]
    fn strip_em_tags_keeps_plain_and_unclosed_text() {
        assert_eq!(strip_em_tags("英雄联盟"), "英雄联盟");
        assert_eq!(strip_em_tags(""), "");
        assert_eq!(strip_em_tags("a<b"), "a<b");
    }

    #[test]
    fn extracts_access_id_from_page_html() {
        let html = r#"<script>window._render_data_={"access_id":"JWT.abc.def","other":1}</script>"#;
        assert_eq!(extract_access_id(html).as_deref(), Some("JWT.abc.def"));

        // 冒号两侧有空白也能认
        let spaced = r#"{"access_id" :  "tok"}"#;
        assert_eq!(extract_access_id(spaced).as_deref(), Some("tok"));

        // 缺字段 / 空值 / 没有冒号都返回 None
        assert_eq!(extract_access_id("<html></html>"), None);
        assert_eq!(extract_access_id(r#"{"access_id":""}"#), None);
        assert_eq!(extract_access_id(r#"{"access_id"}"#), None);
        assert_eq!(extract_access_id(r#"{"access_id":123}"#), None);
    }

    #[test]
    fn area_room_list_reports_next_page() {
        let more = LiveRoomList {
            has_more: 1,
            list: vec![LiveAreaRoom::default()],
            ..Default::default()
        };
        assert!(more.has_next());

        let last = LiveRoomList {
            has_more: 0,
            list: vec![LiveAreaRoom::default()],
            ..Default::default()
        };
        assert!(!last.has_next());

        // has_more 说还有、但列表为空时不算有下一页，避免死循环
        let empty = LiveRoomList {
            has_more: 1,
            list: Vec::new(),
            ..Default::default()
        };
        assert!(!empty.has_next());
    }

    #[test]
    fn area_room_list_deserializes_real_shape() {
        let raw = serde_json::json!({
            "count": 0,
            "has_more": 1,
            "banner": [],
            "list": [{
                "roomid": 3_860_590,
                "uid": 294_709,
                "uname": "野生西瓜",
                "title": "我发现边混边赢会很舒服",
                "online": 54344,
                "area_v2_id": 86,
                "area_v2_name": "英雄联盟",
                "area_v2_parent_id": 2,
                "area_v2_parent_name": "网游",
                "parent_name": null
            }]
        });
        let list: LiveRoomList = serde_json::from_value(raw).unwrap();
        assert_eq!(list.list.len(), 1);
        let room = &list.list[0];
        assert_eq!(room.roomid, 3_860_590);
        assert_eq!(room.area_v2_name, "英雄联盟");
        assert_eq!(room.area_v2_parent_id, 2);
        assert_eq!(room.url(), "https://live.bilibili.com/3860590");
    }
}
