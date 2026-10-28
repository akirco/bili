use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{
    BatchModifyResult, RelationFollowingResult, UserInfo, UserStat, UserVideoListResult,
};
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
