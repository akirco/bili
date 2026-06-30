use crate::client::{BiliClientInner, FormBuilder, Params};
use crate::error::BiliError;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct CommentClient {
    inner: Arc<BiliClientInner>,
}

impl CommentClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn list(
        &self,
        comment_type: i64,
        oid: i64,
        sort: Option<i64>,
        pn: Option<i64>,
        ps: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("type", comment_type.to_string());
        params.push("oid", oid.to_string());
        params.push("sort", sort.unwrap_or(0).to_string());
        params.push("pn", pn.unwrap_or(1).to_string());
        params.push("ps", ps.unwrap_or(20).to_string());

        crate::client::BiliClient { inner: self.inner.clone() }
            .get_raw("https://api.bilibili.com/x/v2/reply/main", params)
            .await
    }

    pub async fn list_wbi(
        &self,
        comment_type: i64,
        oid: i64,
        mode: Option<i64>,
        pagination_str: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = HashMap::new();
        params.insert("type".to_string(), comment_type.to_string());
        params.insert("oid".to_string(), oid.to_string());
        if let Some(m) = mode {
            params.insert("mode".to_string(), m.to_string());
        }
        if let Some(p) = pagination_str {
            params.insert("pagination_str".to_string(), p.to_string());
        }

        crate::client::BiliClient { inner: self.inner.clone() }
            .wbi_get_raw("https://api.bilibili.com/x/v2/reply/wbi/main", params)
            .await
    }

    pub async fn add(
        &self,
        comment_type: i64,
        oid: i64,
        message: &str,
        root: Option<i64>,
        parent: Option<i64>,
        plat: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("message", message)
            .push_opt("root", root.map(|r| r.to_string()))
            .push_opt("parent", parent.map(|p| p.to_string()))
            .push_opt("plat", plat.map(|p| p.to_string()))
            .csrf(csrf)
            .build();

        let result: Option<serde_json::Value> = crate::client::BiliClient { inner: self.inner.clone() }
            .post("https://api.bilibili.com/x/v2/reply/add", &form)
            .await?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    pub async fn like(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/action", &form)
            .await?;
        Ok(())
    }

    pub async fn hate(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/hate", &form)
            .await?;
        Ok(())
    }

    pub async fn delete(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/del", &form)
            .await?;
        Ok(())
    }

    pub async fn top(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        action: Option<i64>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push_opt("action", action.map(|a| a.to_string()))
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/top", &form)
            .await?;
        Ok(())
    }

    pub async fn report(
        &self,
        comment_type: i64,
        oid: i64,
        rpid: i64,
        reason: i64,
        content: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("type", comment_type.to_string())
            .push("oid", oid.to_string())
            .push("rpid", rpid.to_string())
            .push("reason", reason.to_string())
            .push_opt("content", content)
            .csrf(csrf)
            .build();

        crate::client::BiliClient { inner: self.inner.clone() }
            .post::<serde_json::Value>("https://api.bilibili.com/x/v2/reply/report", &form)
            .await?;
        Ok(())
    }
}