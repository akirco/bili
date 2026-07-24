use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{CoinResult, HasCoinResult, HasLikeResult, TripleResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct ActionClient {
    client: BiliClient,
}

impl ActionClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 给视频投币
    ///
    /// `POST https://api.bilibili.com/x/web-interface/coin/add`
    ///
    /// - `aid`: 稿件 avid
    /// - `multiply`: 投币数量 (1 或 2, 默认1)
    /// - `select_like`: 投币时是否同时点赞 (默认不点赞)
    pub async fn coin_video(
        &self,
        aid: i64,
        multiply: Option<i64>,
        select_like: Option<bool>,
    ) -> Result<BiliResponse<Option<CoinResult>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push_val("aid", aid)
            .push_val("multiply", multiply.unwrap_or(1))
            .push_opt(
                "select_like",
                select_like.filter(|&v| v).map(|_| "1".to_string()),
            )
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/web-interface/coin/add", &form)
            .await
    }

    /// 检查是否已给视频投币
    ///
    /// `GET https://api.bilibili.com/x/web-interface/archive/coins`
    ///
    /// - `aid`: 稿件 avid
    pub async fn has_coin(&self, aid: i64) -> Result<BiliResponse<HasCoinResult>, BiliError> {
        let params = Params::new().push_val("aid", aid);
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/archive/coins",
                params,
            )
            .await
    }

    /// 点赞/取消点赞视频
    ///
    /// `POST https://api.bilibili.com/x/web-interface/archive/like`
    ///
    /// - `aid`: 稿件 avid
    /// - `like`: true=点赞, false=取消点赞
    pub async fn like_video(
        &self,
        aid: i64,
        like: bool,
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;
        let like_val = if like { "1" } else { "2" };

        let form = FormBuilder::new()
            .push_val("aid", aid)
            .push("like", like_val)
            .csrf(csrf)
            .build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like",
                &form,
            )
            .await
    }

    /// 检查是否已点赞视频
    ///
    /// `GET https://api.bilibili.com/x/web-interface/archive/has/like`
    ///
    /// - `aid`: 稿件 avid
    pub async fn has_like(&self, aid: i64) -> Result<BiliResponse<HasLikeResult>, BiliError> {
        let params = Params::new().push_val("aid", aid);
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/archive/has/like",
                params,
            )
            .await
    }

    /// 一键三连 (点赞+投币+收藏)
    ///
    /// `POST https://api.bilibili.com/x/web-interface/archive/like/triple`
    ///
    /// - `aid`: 稿件 avid
    pub async fn triple(&self, aid: i64) -> Result<BiliResponse<Option<TripleResult>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new().push_val("aid", aid).csrf(csrf).build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/archive/like/triple",
                &form,
            )
            .await
    }
}
