use crate::client::BiliClient;
use crate::error::BiliError;

impl BiliClient {
    /// Request QR code for scan-to-login
    ///
    /// Calls the Bilibili passport login API to generate a QR code for the user to scan and log in.
    /// The returned JSON contains `data.qrcode_key` (for polling) and `data.url` (QR code content).
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing the QR code URL and qrcode_key
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn login_qrcode_generate(&self) -> Result<serde_json::Value, BiliError> {
        self.get_raw(
            "https://passport.bilibili.com/x/passport-login/web/qrcode/generate",
            &[],
        )
        .await
    }

    /// Poll QR code scan status
    ///
    /// Use the qrcode_key to poll the login status.
    /// On successful login, automatically extracts and saves Cookie info (SESSDATA, bili_jct, DedeUserID) from the response.
    ///
    /// # Arguments
    ///
    /// * `qrcode_key` - QR code key obtained from `login_qrcode_generate`
    ///
    /// # Returns
    ///
    /// Returns the full JSON Value containing login status info
    ///
    /// # Status Codes
    ///
    /// * `86101` - Not scanned
    /// * `86090` - Scanned but not confirmed
    /// * `86038` - QR code expired
    /// * `0` - Login successful
    ///
    /// # Errors
    ///
    /// * `BiliError::Http` - HTTP request failed
    /// * `BiliError::Json` - JSON parsing failed
    pub async fn login_qrcode_poll(
        &self,
        qrcode_key: &str,
    ) -> Result<serde_json::Value, BiliError> {
        let value = self
            .get_raw(
                "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
                &[("qrcode_key", qrcode_key)],
            )
            .await?;

        if value["data"]["code"].as_i64() == Some(0)
            && let Some(url_str) = value["data"]["url"].as_str()
            && !url_str.is_empty()
            && let Ok(url) = url::Url::parse(url_str)
        {
            self.creds.set_from_url(&url).await;
        }

        Ok(value)
    }
}
