use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{CommentAddResult, CommentList, CommentListWbi};
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
