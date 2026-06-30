use crate::client::{BiliClientInner, FormBuilder, Params};
use crate::error::BiliError;
use std::sync::Arc;

#[derive(Clone)]
pub struct FavClient {
    inner: Arc<BiliClientInner>,
}

impl FavClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self { inner }
    }

    pub async fn folder_info(&self, media_id: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("media_id", media_id.to_string());
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw("https://api.bilibili.com/x/v3/fav/folder/info", params)
        .await
    }

    pub async fn folder_list_all(
        &self,
        up_mid: i64,
        r#type: Option<i64>,
        rid: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("up_mid", up_mid.to_string());
        params.push("type", r#type.unwrap_or(2).to_string());
        if let Some(r) = rid {
            params.push("rid", r.to_string());
        }
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
            params,
        )
        .await
    }

    pub async fn collected_list(
        &self,
        up_mid: i64,
        ps: Option<i64>,
        pn: Option<i64>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("up_mid", up_mid.to_string());
        if let Some(p) = ps {
            params.push("ps", p.to_string());
        }
        if let Some(p) = pn {
            params.push("pn", p.to_string());
        }
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/collected/list",
            params,
        )
        .await
    }

    pub async fn resource_infos(
        &self,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("resources", resources);
        if let Some(p) = platform {
            params.push("platform", p);
        }
        let mut resp: serde_json::Value = crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw("https://api.bilibili.com/x/v3/fav/resource/infos", params)
        .await?;
        Ok(serde_json::from_value(resp["data"].take())?)
    }
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
    ) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("media_id", media_id.to_string());
        if let Some(t) = tid {
            params.push("tid", t.to_string());
        }
        if let Some(k) = keyword {
            params.push("keyword", k);
        }
        if let Some(t) = r#type {
            params.push("type", t.to_string());
        }
        params.push("ps", ps.unwrap_or(20).to_string());
        if let Some(p) = pn {
            params.push("pn", p.to_string());
        }
        if let Some(p) = platform {
            params.push("platform", p);
        }
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw("https://api.bilibili.com/x/v3/fav/resource/list", params)
        .await
    }

    pub async fn resource_ids(
        &self,
        media_id: i64,
        platform: Option<&str>,
    ) -> Result<Vec<serde_json::Value>, BiliError> {
        let mut params = Params::new();
        params.push("media_id", media_id.to_string());
        if let Some(p) = platform {
            params.push("platform", p);
        }
        let mut resp: serde_json::Value = crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw("https://api.bilibili.com/x/v3/fav/resource/ids", params)
        .await?;
        Ok(serde_json::from_value(resp["data"].take())?)
    }

    pub async fn folder_add(
        &self,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("title", title)
            .push_opt("intro", intro)
            .push_opt("privacy", privacy.map(|p| p.to_string()))
            .push_opt("cover", cover)
            .csrf(csrf)
            .build();

        let result: Option<serde_json::Value> = crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post("https://api.bilibili.com/x/v3/fav/folder/add", &form)
        .await?;
        Ok(result.unwrap_or(serde_json::Value::Null))
    }

    pub async fn folder_edit(
        &self,
        media_id: i64,
        title: &str,
        intro: Option<&str>,
        privacy: Option<i64>,
        cover: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("media_id", media_id.to_string())
            .push("title", title)
            .push_opt("intro", intro)
            .push_opt("privacy", privacy.map(|p| p.to_string()))
            .push_opt("cover", cover)
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/folder/edit", &form)
        .await?;
        Ok(())
    }

    pub async fn folder_del(&self, media_ids: &str) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("media_ids", media_ids)
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/folder/del", &form)
        .await?;
        Ok(())
    }

    pub async fn resource_copy(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("src_media_id", src_media_id.to_string())
            .push("tar_media_id", tar_media_id.to_string())
            .push("mid", mid.to_string())
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/copy", &form)
        .await?;
        Ok(())
    }

    pub async fn resource_move(
        &self,
        src_media_id: i64,
        tar_media_id: i64,
        mid: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("src_media_id", src_media_id.to_string())
            .push("tar_media_id", tar_media_id.to_string())
            .push("mid", mid.to_string())
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/move", &form)
        .await?;
        Ok(())
    }

    pub async fn resource_batch_del(
        &self,
        media_id: i64,
        resources: &str,
        platform: Option<&str>,
    ) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("media_id", media_id.to_string())
            .push("resources", resources)
            .push_opt("platform", platform)
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>(
            "https://api.bilibili.com/x/v3/fav/resource/batch-del",
            &form,
        )
        .await?;
        Ok(())
    }

    pub async fn resource_clean(&self, media_id: i64) -> Result<(), BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("media_id", media_id.to_string())
            .csrf(csrf)
            .build();

        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post::<serde_json::Value>("https://api.bilibili.com/x/v3/fav/resource/clean", &form)
        .await?;
        Ok(())
    }

    pub async fn add(&self, rid: i64, add_media_ids: i64) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("rid", rid.to_string())
            .push("type", "2")
            .push("add_media_ids", add_media_ids.to_string())
            .csrf(csrf)
            .build();

        let data: Option<serde_json::Value> = crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
        .await?;
        data.ok_or(BiliError::Parse(
            "favorite_add response data missing".into(),
        ))
    }

    pub async fn resource_deal(
        &self,
        rid: i64,
        r#type: i64,
        add_media_ids: Option<&str>,
        del_media_ids: Option<&str>,
    ) -> Result<serde_json::Value, BiliError> {
        let csrf = self.inner.creds.csrf().await?;

        let form = FormBuilder::new()
            .push("rid", rid.to_string())
            .push("type", r#type.to_string())
            .push_opt("add_media_ids", add_media_ids)
            .push_opt("del_media_ids", del_media_ids)
            .csrf(csrf)
            .build();

        let data: Option<serde_json::Value> = crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .post("https://api.bilibili.com/x/v3/fav/resource/deal", &form)
        .await?;
        data.ok_or(BiliError::Parse(
            "fav_resource_deal response data missing".into(),
        ))
    }

    pub async fn has_favorite(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("aid", aid.to_string());
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw("https://api.bilibili.com/x/v2/fav/video/favoured", params)
        .await
    }

    pub async fn folder_list(&self, up_mid: i64) -> Result<serde_json::Value, BiliError> {
        let mut params = Params::new();
        params.push("up_mid", up_mid.to_string());
        crate::client::BiliClient {
            inner: self.inner.clone(),
        }
        .get_raw(
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all",
            params,
        )
        .await
    }
}
