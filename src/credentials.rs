use crate::error::BiliError;
use reqwest::header::HeaderValue;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CredentialsManager {
    inner: Arc<RwLock<CredentialsInner>>,
}

impl Default for CredentialsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct CredentialsInner {
    sessdata: Option<String>,
    bili_jct: Option<String>,
    buvid3: Option<String>,
    dedeuserid: Option<String>,
}

impl CredentialsManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CredentialsInner::default())),
        }
    }

    pub async fn set(&self, sessdata: &str, bili_jct: &str, buvid3: &str) {
        let mut inner = self.inner.write().await;
        inner.sessdata = Some(sessdata.to_string());
        inner.bili_jct = Some(bili_jct.to_string());
        inner.buvid3 = Some(buvid3.to_string());
    }

    pub async fn set_from_url(&self, url: &url::Url) {
        let mut inner = self.inner.write().await;
        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "SESSDATA" => inner.sessdata = Some(v.into_owned()),
                "bili_jct" => inner.bili_jct = Some(v.into_owned()),
                "DedeUserID" => inner.dedeuserid = Some(v.into_owned()),
                _ => {}
            }
        }
    }

    pub async fn get_sessdata(&self) -> Option<String> {
        self.inner.read().await.sessdata.clone()
    }

    pub async fn get_bili_jct(&self) -> Option<String> {
        self.inner.read().await.bili_jct.clone()
    }

    pub async fn get_buvid3(&self) -> Option<String> {
        self.inner.read().await.buvid3.clone()
    }

    pub async fn get_dedeuserid(&self) -> Option<String> {
        self.inner.read().await.dedeuserid.clone()
    }

    pub async fn csrf(&self) -> Result<String, BiliError> {
        self.inner
            .read()
            .await
            .bili_jct
            .clone()
            .ok_or(BiliError::CsrfNotFound)
    }

    pub async fn export(
        &self,
    ) -> (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        let inner = self.inner.read().await;
        (
            inner.sessdata.clone(),
            inner.bili_jct.clone(),
            inner.buvid3.clone(),
            inner.dedeuserid.clone(),
        )
    }

    pub async fn import(
        &self,
        sessdata: Option<String>,
        bili_jct: Option<String>,
        buvid3: Option<String>,
        dedeuserid: Option<String>,
    ) {
        let mut inner = self.inner.write().await;
        if let Some(v) = sessdata {
            inner.sessdata = Some(v);
        }
        if let Some(v) = bili_jct {
            inner.bili_jct = Some(v);
        }
        if let Some(v) = buvid3 {
            inner.buvid3 = Some(v);
        }
        if let Some(v) = dedeuserid {
            inner.dedeuserid = Some(v);
        }
    }

    pub(crate) async fn build_cookie_header(&self) -> Option<HeaderValue> {
        let inner = self.inner.read().await;
        let mut cookies = Vec::new();
        if let Some(ref s) = inner.sessdata {
            cookies.push(format!("SESSDATA={}", s));
        }
        if let Some(ref b) = inner.bili_jct {
            cookies.push(format!("bili_jct={}", b));
        }
        if let Some(ref b) = inner.buvid3 {
            cookies.push(format!("buvid3={}", b));
        }
        if let Some(ref d) = inner.dedeuserid {
            cookies.push(format!("DedeUserID={}", d));
        }
        if cookies.is_empty() {
            None
        } else {
            HeaderValue::from_str(&cookies.join("; ")).ok()
        }
    }
}
