use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{
    EmptyDataResult, FavDealResult, FavFolderInfo, FavFolderListResult, FavHasFavoriteResult,
    FavResource, FavResourceId, FavResourceListResult,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct FavClient {
    client: BiliClient,
}

impl FavClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 获取收藏夹详细信息
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/folder/info`
    ///
    /// - `media_id`: 收藏夹 id
    pub async fn folder_info(
        &self,
        media_id: i64,
    ) -> Result<BiliResponse<FavFolderInfo>, BiliError> {
        let params = Params::new().push_val("media_id", media_id);
        self.client
            .get("https://api.bilibili.com/x/v3/fav/folder/info", params)
            .await
    }

    /// 获取用户创建的收藏夹列表
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/folder/created/list-all`
    ///
    /// - `up_mid`: 目标用户 mid
    /// - `r#type`: 资源类型 (默认2=视频)
    /// - `rid`: 资源avid (可选)
    pub async fn folder_list_all(
        &self,
        up_mid: i64,
        r#type: Option<i64>,
        rid: Option<i64>,
    ) -> Result<BiliResponse<FavFolderListResult>, BiliError> {
        let mut params = Params::new()
            .push_val("up_mid", up_mid)
            .push_val("type", r#type.unwrap_or(2));
        if let Some(r) = rid {
            params = params.push_val("rid", r);
        }
        self.client
            .get(
                "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
                params,
            )
            .await
    }

    /// 获取用户收藏的收藏夹列表
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/folder/collected/list`
    ///
    /// - `up_mid`: 目标用户 mid
    /// - `ps`: 每页项数
    /// - `pn`: 页码
    pub async fn collected_list(
        &self,
        up_mid: i64,
        ps: Option<i64>,
        pn: Option<i64>,
    ) -> Result<BiliResponse<FavFolderListResult>, BiliError> {
        let mut params = Params::new().push_val("up_mid", up_mid);
        if let Some(p) = ps {
            params = params.push_val("ps", p);
        }
        if let Some(p) = pn {
            params = params.push_val("pn", p);
        }
        self.client
            .get(
                "https://api.bilibili.com/x/v3/fav/folder/collected/list",
                params,
            )
            .await
    }

    /// 批量获取收藏资源信息
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/resource/infos`
    ///
    /// - `resources`: 资源列表 (格式: "avid:type,avid:type,...")
    /// - `platform`: 平台
    pub async fn resource_infos(
        &self,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<Vec<FavResource>, BiliError> {
        let mut params = Params::new().push("resources", resources);
        if let Some(p) = platform {
            params = params.push("platform", p);
        }
        let resp: serde_json::Value = self
            .client
            .get_raw("https://api.bilibili.com/x/v3/fav/resource/infos", params)
            .await?;
        Ok(serde_json::from_value(resp["data"].clone())?)
    }

    /// 获取收藏夹内容列表
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/resource/list`
    ///
    /// - `media_id`: 收藏夹 id
    /// - `tid`: 分区 tid
    /// - `keyword`: 关键词
    /// - `r#type`: 资源类型
    /// - `ps`: 每页项数 (默认20)
    /// - `pn`: 页码
    /// - `platform`: 平台
    #[allow(clippy::too_many_arguments)]
    pub async fn resource_list(
        &self,
        media_id: i64,
        tid: Option<i64>,
        keyword: Option<&str>,
        r#type: Option<i64>,
        ps: Option<i64>,
        pn: Option<i64>,
        platform: Option<&str>,
    ) -> Result<BiliResponse<FavResourceListResult>, BiliError> {
        let mut params = Params::new().push_val("media_id", media_id);
        if let Some(t) = tid {
            params = params.push_val("tid", t);
        }
        if let Some(k) = keyword {
            params = params.push("keyword", k);
        }
        if let Some(t) = r#type {
            params = params.push_val("type", t);
        }
        params = params.push_val("ps", ps.unwrap_or(20));
        if let Some(p) = pn {
            params = params.push_val("pn", p);
        }
        if let Some(p) = platform {
            params = params.push("platform", p);
        }
        self.client
            .get("https://api.bilibili.com/x/v3/fav/resource/list", params)
            .await
    }

    /// 获取收藏资源 id 列表
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/resource/ids`
    ///
    /// - `media_id`: 收藏夹 id
    /// - `platform`: 平台
    pub async fn resource_ids(
        &self,
        media_id: i64,
        platform: Option<&str>,
    ) -> Result<Vec<FavResourceId>, BiliError> {
        let mut params = Params::new().push_val("media_id", media_id);
        if let Some(p) = platform {
            params = params.push("platform", p);
        }
        let resp: serde_json::Value = self
            .client
            .get_raw("https://api.bilibili.com/x/v3/fav/resource/ids", params)
            .await?;
        Ok(serde_json::from_value(resp["data"].clone())?)
    }

    /// 创建收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/folder/add`
    ///
    /// - `title`: 收藏夹标题
    /// - `intro`: 收藏夹简介
    /// - `privacy`: 是否私密 (0=公开, 1=私密)
    /// - `cover`: 收藏夹封面 url
    pub async fn folder_add(
        &self,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<BiliResponse<FavFolderInfo>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push("title", title)
            .push_opt("intro", intro)
            .push_opt_val("privacy", privacy)
            .push_opt("cover", cover)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/folder/add", &form)
            .await
    }

    /// 编辑收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/folder/edit`
    ///
    /// - `media_id`: 收藏夹 id
    /// - `title`: 收藏夹标题
    /// - `intro`: 收藏夹简介
    /// - `privacy`: 是否私密 (0=公开, 1=私密)
    /// - `cover`: 收藏夹封面 url
    pub async fn folder_edit(
        &self,
        media_id: i64,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<BiliResponse<FavFolderInfo>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("media_id", media_id)
            .push("title", title)
            .push_opt("intro", intro)
            .push_opt_val("privacy", privacy)
            .push_opt("cover", cover)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/folder/edit", &form)
            .await
    }

    /// 删除收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/folder/del`
    ///
    /// - `media_ids`: 收藏夹 id 列表 (逗号分隔)
    pub async fn folder_del(
        &self,
        media_ids: &str,
    ) -> Result<BiliResponse<EmptyDataResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push("media_ids", media_ids)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/folder/del", &form)
            .await
    }

    /// 复制收藏资源到另一个收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/copy`
    ///
    /// - `src_media_id`: 源收藏夹 id
    /// - `tar_media_id`: 目标收藏夹 id
    /// - `mid`: 用户 mid
    /// - `resources`: 资源列表 (格式: "avid:type,avid:type,...")
    /// - `platform`: 平台
    pub async fn resource_copy(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<BiliResponse<EmptyDataResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("src_media_id", src_media_id)
            .push_val("tar_media_id", tar_media_id)
            .push_val("mid", mid)
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/resource/copy", &form)
            .await
    }

    /// 移动收藏资源到另一个收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/move`
    ///
    /// - `src_media_id`: 源收藏夹 id
    /// - `tar_media_id`: 目标收藏夹 id
    /// - `mid`: 用户 mid
    /// - `resources`: 资源列表 (格式: "avid:type,avid:type,...")
    /// - `platform`: 平台
    pub async fn resource_move(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<BiliResponse<EmptyDataResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("src_media_id", src_media_id)
            .push_val("tar_media_id", tar_media_id)
            .push_val("mid", mid)
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/resource/move", &form)
            .await
    }

    /// 批量删除收藏资源
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/batch-del`
    ///
    /// - `media_id`: 收藏夹 id
    /// - `resources`: 资源列表 (格式: "avid:type,avid:type,...")
    /// - `platform`: 平台
    pub async fn resource_batch_del(
        &self,
        media_id: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<BiliResponse<EmptyDataResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("media_id", media_id)
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        self.client
            .post(
                "https://api.bilibili.com/x/v3/fav/resource/batch-del",
                &form,
            )
            .await
    }

    /// 清空收藏夹内容
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/clean`
    ///
    /// - `media_id`: 收藏夹 id
    pub async fn resource_clean(
        &self,
        media_id: i64,
    ) -> Result<BiliResponse<EmptyDataResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("media_id", media_id)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/resource/clean", &form)
            .await
    }

    /// 添加视频到收藏夹
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/deal`
    ///
    /// - `rid`: 视频 avid
    /// - `add_media_ids`: 目标收藏夹 id 列表 (逗号分隔)
    pub async fn add(
        &self,
        rid: i64,
        add_media_ids: i64,
    ) -> Result<BiliResponse<FavDealResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("rid", rid)
            .push("type", "2")
            .push_val("add_media_ids", add_media_ids)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
            .await
    }

    /// 收藏资源操作 (添加/移除)
    ///
    /// `POST https://api.bilibili.com/x/v3/fav/resource/deal`
    ///
    /// - `rid`: 资源 id (视频 avid)
    /// - `r#type`: 资源类型 (2=视频)
    /// - `add_media_ids`: 添加到的收藏夹 id 列表 (逗号分隔)
    /// - `del_media_ids`: 移除的收藏夹 id 列表 (逗号分隔)
    pub async fn resource_deal(
        &self,
        rid: i64,
        r#type: i64,
        add_media_ids: Option<&str>,
        del_media_ids: Option<&str>,
    ) -> Result<BiliResponse<FavDealResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("rid", rid)
            .push_val("type", r#type)
            .push_opt("add_media_ids", add_media_ids)
            .push_opt("del_media_ids", del_media_ids)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
            .await
    }

    /// 检查视频是否已收藏
    ///
    /// `GET https://api.bilibili.com/x/v2/fav/video/favoured`
    ///
    /// - `aid`: 视频 avid
    pub async fn has_favorite(
        &self,
        aid: i64,
    ) -> Result<BiliResponse<FavHasFavoriteResult>, BiliError> {
        let params = Params::new().push_val("aid", aid);
        self.client
            .get("https://api.bilibili.com/x/v2/fav/video/favoured", params)
            .await
    }

    /// 获取用户创建的收藏夹列表 (简化版)
    ///
    /// `GET https://api.bilibili.com/x/v3/fav/folder/created/list-all`
    ///
    /// - `up_mid`: 目标用户 mid
    pub async fn folder_list(
        &self,
        up_mid: i64,
    ) -> Result<BiliResponse<FavFolderListResult>, BiliError> {
        let params = Params::new().push_val("up_mid", up_mid);
        self.client
            .get(
                "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
                params,
            )
            .await
    }
}
