use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{HistoryResult, ToviewResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct HistoryClient {
    client: BiliClient,
}

impl HistoryClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取观看历史记录
    ///
    /// `GET https://api.bilibili.com/x/web-interface/history/cursor`
    ///
    /// - `pn`: 页码 (默认1)
    /// - `ps`: 每页项数 (默认20)
    pub async fn list(
        &self,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<BiliResponse<HistoryResult>, BiliError> {
        let params = Params::new()
            .push_val("pn", pn.unwrap_or(1))
            .push_val("ps", ps.unwrap_or(20));
        self.client
            .get(
                "https://api.bilibili.com/x/web-interface/history/cursor",
                params,
            )
            .await
    }

    /// 删除观看历史记录
    ///
    /// `POST https://api.bilibili.com/x/web-interface/history/del`
    ///
    /// - `aid`: 视频 avid
    pub async fn delete(
        &self,
        aid: i64,
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push_val("aid", aid)
            .push("history_type", "archive")
            .push("type", "del")
            .csrf(csrf)
            .build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/history/del",
                &form,
            )
            .await
    }

    /// 清空观看历史记录
    ///
    /// `POST https://api.bilibili.com/x/web-interface/history/clear`
    pub async fn clear(&self) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new().push("type", "clear").csrf(csrf).build();

        self.client
            .post(
                "https://api.bilibili.com/x/web-interface/history/clear",
                &form,
            )
            .await
    }

    /// 获取稍后再看列表
    ///
    /// `GET https://api.bilibili.com/x/v2/history/toview`
    pub async fn toview_list(&self) -> Result<BiliResponse<ToviewResult>, BiliError> {
        self.client
            .get(
                "https://api.bilibili.com/x/v2/history/toview",
                Params::new(),
            )
            .await
    }

    /// 添加视频到稍后再看
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/add`
    ///
    /// - `aid`: 视频 avid (与 bvid 任选一个)
    /// - `bvid`: 视频 bvid (与 aid 任选一个)
    pub async fn toview_add(
        &self,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let mut form = FormBuilder::new().csrf(csrf);
        if let Some(aid) = aid {
            form = form.push_val("aid", aid);
        }
        if let Some(bvid) = bvid {
            form = form.push("bvid", bvid);
        }

        self.client
            .post(
                "https://api.bilibili.com/x/v2/history/toview/add",
                &form.build(),
            )
            .await
    }

    /// 从稍后再看中删除视频
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/del`
    ///
    /// - `aid`: 视频 avid (删除指定记录)
    /// - `viewed`: 是否删除已观看的 (true=删除所有已观看的)
    pub async fn toview_del(
        &self,
        aid: Option<i64>,
        viewed: Option<bool>,
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let mut form = FormBuilder::new().csrf(csrf);
        if let Some(aid) = aid {
            form = form.push_val("aid", aid);
        }
        if let Some(true) = viewed {
            form = form.push("viewed", "true");
        }

        self.client
            .post(
                "https://api.bilibili.com/x/v2/history/toview/del",
                &form.build(),
            )
            .await
    }

    /// 清空稍后再看列表
    ///
    /// `POST https://api.bilibili.com/x/v2/history/toview/clear`
    pub async fn toview_clear(&self) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new().csrf(csrf).build();

        self.client
            .post("https://api.bilibili.com/x/v2/history/toview/clear", &form)
            .await
    }
}
