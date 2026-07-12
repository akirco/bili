use crate::client::{BiliClient, BiliClientInner, FormBuilder, BiliResponse, Params};
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

    /// 获取评论列表
    ///
    /// `GET https://api.bilibili.com/x/v2/reply/main`
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
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("sort", sort.unwrap_or(0).to_string())
            .push("pn", pn.unwrap_or(1).to_string())
            .push("ps", ps.unwrap_or(20).to_string());
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
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string());
        if let Some(m) = mode {
            params = params.push("mode", m.to_string());
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
    ) -> Result<BiliResponse<Option<CommentAddResult>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("message", message)
            .push_opt("root", root.map(|r| r.to_string()))
            .push_opt("parent", parent.map(|p| p.to_string()))
            .push_opt("plat", plat.map(|p| p.to_string()))
            .csrf(csrf)
            .build();

        self.client
            .post_required("https://api.bilibili.com/x/v2/reply/add", &form)
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
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
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
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
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
    pub async fn delete(&self, comment_type: i64, oid: i64, rpid: i64) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
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
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
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
    ) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push("reason", reason.to_string())
            .push_opt("content", content)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v2/reply/report", &form)
            .await
    }
}
