use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::{SearchAllResult, SearchHotWordResult, SearchSquareResult, SearchTypeResult};
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
    ///   - media_bangumi: 番剧
    ///   - media_ft: 影视
    ///   - live: 直播间及主播
    ///   - live_room: 直播间
    ///   - live_user: 主播
    ///   - article: 专栏
    ///   - topic: 话题
    ///   - bili_user: 用户
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
    /// `GET https://api.bilibili.com/x/web-interface/wbi/search/type` (search_type=bili_user)
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
