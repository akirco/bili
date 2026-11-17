use crate::client::{BiliClient, BiliClientInner, BiliResponse, Params};
use crate::error::BiliError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct LoginClient {
    client: BiliClient,
}

impl LoginClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// 生成登录二维码
    ///
    /// `GET https://passport.bilibili.com/x/passport-login/web/qrcode/generate`
    pub async fn qrcode(&self) -> Result<BiliResponse<LoginQrcodeResult>, BiliError> {
        let params = Params::new().push("source", "main-fe-header");
        self.client
            .get(
                "https://passport.bilibili.com/x/passport-login/web/qrcode/generate",
                params,
            )
            .await
    }

    /// 轮询二维码登录状态
    ///
    /// `GET https://passport.bilibili.com/x/passport-login/web/qrcode/poll`
    ///
    /// - `qrcode_key`: 二维码 key (从 `qrcode()` 返回)
    pub async fn qrcode_status(
        &self,
        qrcode_key: &str,
    ) -> Result<BiliResponse<LoginQrcodeStatusResult>, BiliError> {
        let params = Params::new().push("qrcode_key", qrcode_key);
        self.client
            .get(
                "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
                params,
            )
            .await
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LoginQrcodeResult {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LoginQrcodeStatusResult {
    pub code: i64,
    pub message: String,
    pub url: Option<String>,
    pub refresh_token: Option<String>,
    pub timestamp: Option<i64>,
}
