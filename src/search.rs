use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchClient {
    client: BiliClient,
}

impl SearchClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 综合搜索 (web端, Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/all/v2`
    ///
    /// 返回和关键字相关的20条信息
    ///
    /// - `keyword`: 需要搜索的关键词
    /// - `page`: 页码
    /// - `page_size`: 每页项数 (默认20)
    pub async fn all(
        &self,
        keyword: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<BiliResponse<SearchAllResult>, BiliError> {
        let mut params = Params::new();
        if let Some(p) = page {
            params = params.push_val("page", p);
        }
        if let Some(ps) = page_size {
            params = params.push_val("page_size", ps);
        }
        let params = params.push("keyword", keyword);
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/wbi/search/all/v2",
                params,
            )
            .await
    }

    /// 分类搜索 (web端, Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/type`
    ///
    /// - `keyword`: 需要搜索的关键词
    /// - `page`: 页码
    /// - `page_size`: 每页项数 (默认20)
    /// - `search_type`: 搜索目标类型
    ///   - video: 视频
    ///   - `media_bangumi`: 番剧
    ///   - `media_ft`: 影视
    ///   - live: 直播间及主播
    ///   - `live_room`: 直播间
    ///   - `live_user`: 主播
    ///   - article: 专栏
    ///   - topic: 话题
    ///   - `bili_user`: 用户
    ///   - photo: 相簿
    pub async fn query(
        &self,
        keyword: &str,
        page: Option<i64>,
        page_size: Option<i64>,
        search_type: &str,
    ) -> Result<BiliResponse<SearchTypeResult>, BiliError> {
        let mut params = Params::new();
        if let Some(p) = page {
            params = params.push_val("page", p);
        }
        if let Some(ps) = page_size {
            params = params.push_val("page_size", ps);
        }
        let params = params
            .push("search_type", search_type)
            .push("keyword", keyword);

        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/wbi/search/type",
                params,
            )
            .await
    }

    /// 搜索用户
    ///
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/type` (`search_type=bili_user`)
    ///
    /// - `keyword`: 需要搜索的关键词
    /// - `page`: 页码
    /// - `page_size`: 每页项数 (默认20)
    pub async fn user(
        &self,
        keyword: &str,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<BiliResponse<SearchTypeResult>, BiliError> {
        let mut params = Params::new()
            .push("search_type", "bili_user")
            .push("keyword", keyword);
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

    /// 获取热搜列表 (Wbi签名)
    ///
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/square`
    ///
    /// - `limit`: 结果限制 (范围 [1, 50])
    pub async fn hot(&self, limit: i64) -> Result<BiliResponse<SearchSquareResult>, BiliError> {
        let params = Params::new()
            .push_val("limit", limit)
            .push("platform", "web");
        self.client
            .wbi_get(
                "https://api.bilibili.com/x/web-interface/wbi/search/square",
                params,
            )
            .await
    }

    /// 获取热搜列表 (web端)
    ///
    /// `GET https://s.search.bilibili.com/main/hotword`
    ///
    /// 榜单每隔固定时间统计一次，请求后返回搜索前10的关键词
    pub async fn hot_word(&self) -> Result<BiliResponse<SearchHotWordResult>, BiliError> {
        self.client
            .get("https://s.search.bilibili.com/main/hotword", Params::new())
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchVideo {
    pub r#type: String,
    pub id: i64,
    pub author: String,
    pub mid: i64,
    pub typeid: String,
    pub typename: String,
    pub arcurl: String,
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub description: String,
    pub arcrank: String,
    pub pic: String,
    pub play: i64,
    pub video_review: i64,
    pub favorites: i64,
    pub tag: String,
    pub review: i64,
    pub pubdate: i64,
    pub senddate: i64,
    pub duration: String,
    pub badgepay: bool,
    pub hit_columns: Vec<String>,
    pub view_type: String,
    pub is_pay: i64,
    pub is_union_video: i64,
    pub rank_score: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchMediaScore {
    pub user_count: i64,
    pub score: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchMediaDisplayInfo {
    pub bg_color_night: String,
    pub text: String,
    pub border_color: String,
    pub bg_style: i64,
    pub text_color: String,
    pub bg_color: String,
    pub text_color_night: String,
    pub border_color_night: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchMediaBadge {
    pub text: String,
    pub text_color: String,
    pub text_color_night: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub border_color: String,
    pub border_color_night: String,
    pub bg_style: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchMediaEp {
    pub id: i64,
    pub cover: String,
    pub title: String,
    pub url: String,
    pub release_date: String,
    pub badges: Vec<SearchMediaBadge>,
    pub index_title: String,
    pub long_title: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchMedia {
    pub r#type: String,
    pub media_id: i64,
    pub season_id: i64,
    pub title: String,
    pub org_title: String,
    pub cover: String,
    pub media_type: i64,
    pub areas: String,
    pub styles: String,
    pub cv: String,
    pub staff: String,
    pub play_state: i64,
    pub goto_url: String,
    pub desc: String,
    pub corner: i64,
    pub pubtime: i64,
    pub media_mode: i64,
    pub is_avid: bool,
    pub fix_pubtime_str: String,
    pub media_score: Option<SearchMediaScore>,
    pub hit_columns: Option<Vec<String>>,
    pub angle_title: String,
    pub angle_color: i64,
    pub display_info: Vec<SearchMediaDisplayInfo>,
    pub hit_epids: String,
    pub pgc_season_id: i64,
    pub season_type: i64,
    pub season_type_name: String,
    pub selection_style: String,
    pub ep_size: i64,
    pub url: String,
    pub button_text: String,
    pub is_follow: i64,
    pub is_selection: i64,
    pub eps: Vec<SearchMediaEp>,
    pub badges: Vec<SearchMediaBadge>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchLiveRoom {
    pub rank_offset: i64,
    pub uid: i64,
    pub tags: String,
    pub hit_columns: Vec<String>,
    pub live_time: String,
    pub cate_name: String,
    pub live_status: i64,
    pub uname: String,
    pub uface: String,
    pub user_cover: String,
    pub short_id: i64,
    pub area: i64,
    pub r#type: String,
    pub title: String,
    pub cover: String,
    pub online: i64,
    pub rank_index: i64,
    pub rank_score: i64,
    pub roomid: i64,
    pub attentions: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchLiveUser {
    pub rank_offset: i64,
    pub uid: i64,
    pub tags: String,
    pub r#type: String,
    pub live_time: String,
    pub hit_columns: Vec<String>,
    pub live_status: i64,
    pub area: i64,
    pub is_live: bool,
    pub uname: String,
    pub uface: String,
    pub rank_index: i64,
    pub rank_score: i64,
    pub roomid: i64,
    pub attentions: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchArticle {
    pub pub_time: i64,
    pub like: i64,
    pub title: String,
    pub rank_offset: i64,
    pub mid: i64,
    pub image_urls: Vec<String>,
    pub template_id: i64,
    pub category_id: i64,
    pub view: i64,
    pub reply: i64,
    pub rank_index: i64,
    pub desc: String,
    pub rank_score: i64,
    pub r#type: String,
    pub id: i64,
    pub category_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchTopic {
    pub description: String,
    pub pubdate: i64,
    pub title: String,
    pub favourite: i64,
    pub hit_columns: Vec<String>,
    pub review: i64,
    pub rank_offset: i64,
    pub cover: String,
    pub update: i64,
    pub mid: i64,
    pub click: i64,
    pub tp_type: i64,
    pub keyword: String,
    pub tp_id: i64,
    pub rank_index: i64,
    pub author: String,
    pub arcurl: String,
    pub rank_score: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchUserResVideo {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub pubdate: i64,
    pub arcurl: String,
    pub pic: String,
    pub play: serde_json::Value,
    pub dm: serde_json::Value,
    pub coin: i64,
    pub fav: i64,
    pub desc: String,
    pub duration: String,
    pub is_pay: i64,
    pub is_union_video: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchUserOfficialVerify {
    pub r#type: i64,
    pub desc: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchUser {
    pub r#type: String,
    pub mid: i64,
    pub uname: String,
    pub usign: String,
    pub fans: i64,
    pub videos: i64,
    pub upic: String,
    pub verify_info: String,
    pub level: i64,
    pub gender: i64,
    pub is_upuser: i64,
    pub is_live: i64,
    pub room_id: i64,
    pub res: Vec<SearchUserResVideo>,
    pub official_verify: SearchUserOfficialVerify,
    pub hit_columns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchPhoto {
    pub count: i64,
    pub like: i64,
    pub title: String,
    pub hit_columns: Vec<String>,
    pub rank_offset: i64,
    pub cover: String,
    pub mid: i64,
    pub uname: String,
    pub rank_index: i64,
    pub view: i64,
    pub r#type: String,
    pub id: i64,
    pub rank_score: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SearchResultItem {
    Video(SearchVideo),
    MediaBangumi(SearchMedia),
    MediaFt(SearchMedia),
    LiveRoom(SearchLiveRoom),
    LiveUser(SearchLiveUser),
    Article(SearchArticle),
    Topic(SearchTopic),
    BiliUser(SearchUser),
    Photo(SearchPhoto),
    /// Result types added by Bilibili after this crate was written. Kept as
    /// raw JSON instead of failing the whole response.
    Other(serde_json::Value),
}

impl<'de> serde::Deserialize<'de> for SearchResultItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
        let kind = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let parsed = match kind {
            "video" => serde_json::from_value(value).map(Self::Video),
            "media_bangumi" => serde_json::from_value(value).map(Self::MediaBangumi),
            "media_ft" => serde_json::from_value(value).map(Self::MediaFt),
            "live_room" => serde_json::from_value(value).map(Self::LiveRoom),
            "live_user" => serde_json::from_value(value).map(Self::LiveUser),
            "article" => serde_json::from_value(value).map(Self::Article),
            "topic" => serde_json::from_value(value).map(Self::Topic),
            "bili_user" => serde_json::from_value(value).map(Self::BiliUser),
            "photo" => serde_json::from_value(value).map(Self::Photo),
            _ => return Ok(Self::Other(value)),
        };
        parsed.map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchHotWordItem {
    pub status: String,
    pub hot_id: i64,
    pub keyword: String,
    pub resource_id: i64,
    pub goto_type: i64,
    pub res: Vec<serde_json::Value>,
    pub show_name: String,
    pub pos: i64,
    pub word_type: i64,
    pub id: i64,
    pub goto_value: String,
    pub live_id: Vec<serde_json::Value>,
    pub name_type: String,
    pub icon: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchHotWordResult {
    pub exp_str: String,
    pub cost: serde_json::Value,
    pub seid: String,
    pub timestamp: i64,
    pub list: Vec<SearchHotWordItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchSquareTrendingItem {
    pub keyword: String,
    pub show_name: String,
    pub icon: String,
    pub uri: String,
    pub goto: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchSquareTrending {
    pub title: String,
    pub trackid: String,
    pub list: Vec<SearchSquareTrendingItem>,
    pub top_list: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchSquareResult {
    pub trending: SearchSquareTrending,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchAllResult {
    pub seid: String,
    pub page: i64,
    /// 接口字段名是小写的 `pagesize`
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
    pub top_tlist: serde_json::Value,
    pub show_column: i64,
    pub show_module_list: Vec<String>,
    pub result: Vec<SearchResultItem>,
}

/// 分类搜索 API 响应
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct SearchTypeResult {
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
    pub result: serde_json::Value,
    pub show_column: i64,
}
