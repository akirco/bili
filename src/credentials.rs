use crate::error::BiliError;
use reqwest::header::HeaderValue;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Credentials for Bilibili authentication
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Credentials {
    pub sessdata: Option<String>,
    pub bili_jct: Option<String>,
    pub buvid3: Option<String>,
    pub dedeuserid: Option<String>,
}

#[derive(Clone)]
pub struct CredentialsManager {
    inner: Arc<RwLock<Credentials>>,
}

impl Default for CredentialsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialsManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Credentials::default())),
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

    pub async fn export(&self) -> Credentials {
        self.inner.read().await.clone()
    }

    pub async fn import_from(&self, creds: &Credentials) {
        let mut inner = self.inner.write().await;
        if let Some(v) = &creds.sessdata {
            inner.sessdata = Some(v.clone());
        }
        if let Some(v) = &creds.bili_jct {
            inner.bili_jct = Some(v.clone());
        }
        if let Some(v) = &creds.buvid3 {
            inner.buvid3 = Some(v.clone());
        }
        if let Some(v) = &creds.dedeuserid {
            inner.dedeuserid = Some(v.clone());
        }
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

    pub(crate) async fn build_cookie_header(&self) -> Result<Option<HeaderValue>, BiliError> {
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
            Ok(None)
        } else {
            let header = HeaderValue::from_str(&cookies.join("; "))
                .map_err(|e| BiliError::CookieBuild(e.to_string()))?;
            Ok(Some(header))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn empty_credentials_returns_none() {
        let mgr = CredentialsManager::new();
        assert!(mgr.build_cookie_header().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn sessdata_only() {
        let mgr = CredentialsManager::new();
        mgr.set("test_sessdata", "test_jct", "test_buvid3").await;
        let header = mgr.build_cookie_header().await.unwrap().unwrap();
        let val = header.to_str().unwrap();
        assert!(val.contains("SESSDATA=test_sessdata"));
    }

    #[tokio::test]
    async fn all_cookies_present() {
        let mgr = CredentialsManager::new();
        mgr.set("sd", "bj", "bv").await;
        mgr.import(None, None, None, Some("12345".to_string()))
            .await;
        let header = mgr.build_cookie_header().await.unwrap().unwrap();
        let val = header.to_str().unwrap();
        assert!(val.contains("SESSDATA=sd"));
        assert!(val.contains("bili_jct=bj"));
        assert!(val.contains("buvid3=bv"));
        assert!(val.contains("DedeUserID=12345"));
    }

    #[tokio::test]
    async fn export_import_roundtrip() {
        let mgr = CredentialsManager::new();
        mgr.set("sd", "bj", "bv").await;
        mgr.import(None, None, None, Some("uid".to_string()))
            .await;

        let exported = mgr.export().await;
        assert_eq!(exported.sessdata.as_deref(), Some("sd"));
        assert_eq!(exported.bili_jct.as_deref(), Some("bj"));
        assert_eq!(exported.buvid3.as_deref(), Some("bv"));
        assert_eq!(exported.dedeuserid.as_deref(), Some("uid"));

        let mgr2 = CredentialsManager::new();
        mgr2.import_from(&exported).await;
        let header2 = mgr2.build_cookie_header().await.unwrap().unwrap();
        let val2 = header2.to_str().unwrap();
        assert!(val2.contains("SESSDATA=sd"));
        assert!(val2.contains("DedeUserID=uid"));
    }

    #[tokio::test]
    async fn csrf_returns_bili_jct() {
        let mgr = CredentialsManager::new();
        assert!(mgr.csrf().await.is_err());

        mgr.set("sd", "my_csrf", "bv").await;
        assert_eq!(mgr.csrf().await.unwrap(), "my_csrf");
    }

    #[tokio::test]
    async fn individual_getters() {
        let mgr = CredentialsManager::new();
        mgr.set("sd", "bj", "bv").await;
        mgr.import(None, None, None, Some("uid".to_string()))
            .await;

        assert_eq!(mgr.get_sessdata().await.as_deref(), Some("sd"));
        assert_eq!(mgr.get_bili_jct().await.as_deref(), Some("bj"));
        assert_eq!(mgr.get_buvid3().await.as_deref(), Some("bv"));
        assert_eq!(mgr.get_dedeuserid().await.as_deref(), Some("uid"));
    }

    #[tokio::test]
    async fn import_partial_does_not_overwrite_none() {
        let mgr = CredentialsManager::new();
        mgr.set("original_sd", "original_bj", "original_bv").await;

        mgr.import(None, Some("new_bj".to_string()), None, None)
            .await;

        assert_eq!(mgr.get_sessdata().await.as_deref(), Some("original_sd"));
        assert_eq!(mgr.get_bili_jct().await.as_deref(), Some("new_bj"));
        assert_eq!(mgr.get_buvid3().await.as_deref(), Some("original_bv"));
    }
}