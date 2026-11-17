//! Identity: cookie parsing and persistence, CSRF token, login state, buvid.

use super::{BUVID_SPI_URL, BiliClient, Params, parse_buvid};
use crate::error::BiliError;

impl BiliClient {
    /// Flush pending cookie changes to disk.
    ///
    /// Normally this happens in `CookieStore::drop`, but the CLI calls
    /// `std::process::exit` on API errors, which skips destructors.
    pub fn flush_cookies(&self) {
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.flush();
        }
    }

    /// Parse cookies from a URL's query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::Parse`] if `url` is not a valid URL.
    pub fn parse_cookies(&self, url: &str) -> Result<(), BiliError> {
        let parsed = reqwest::Url::parse(url).map_err(|e| BiliError::Parse(e.to_string()))?;
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.set_from_url(&parsed);
        }
        Ok(())
    }

    /// Parse a `Cookie` header string (`SESSDATA=x; bili_jct=y`) into the store.
    pub fn parse_cookie_string(&self, cookies: &str) {
        if let Ok(mut store) = self.inner.cookie_store.lock() {
            store.set_from_cookie_string(cookies);
        }
    }

    /// Get CSRF token from cookie store.
    ///
    /// # Errors
    ///
    /// Returns [`BiliError::CsrfNotFound`] when not logged in or `bili_jct`
    /// is missing, or [`BiliError::CookieBuild`] if the store lock is poisoned.
    pub fn csrf(&self) -> Result<String, BiliError> {
        let token = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .csrf_token()
            .to_string();
        if token.is_empty() {
            Err(BiliError::CsrfNotFound)
        } else {
            Ok(token)
        }
    }

    /// Check if the user is logged in.
    #[must_use]
    pub fn is_logged_in(&self) -> bool {
        self.inner
            .cookie_store
            .lock()
            .is_ok_and(|s| s.is_logged_in())
    }

    /// Ensure the device cookies `buvid3`/`buvid4` exist.
    ///
    /// Bilibili's risk control rejects some endpoints with HTTP 412 when these
    /// cookies are missing. This is best-effort: if the endpoint fails, callers
    /// can still try their actual request.
    ///
    /// # Errors
    ///
    /// Propagates HTTP/parse errors from the fingerprint endpoint, or
    /// [`BiliError::CookieBuild`] if the cookie store lock is poisoned.
    pub async fn bootstrap_buvid(&self) -> Result<(), BiliError> {
        let has_buvid3 = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .has("buvid3");
        if has_buvid3 {
            return Ok(());
        }

        let raw = self.get_raw(BUVID_SPI_URL, Params::new()).await?;
        let Some((buvid3, buvid4)) = parse_buvid(&raw) else {
            return Ok(());
        };
        let mut store = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?;
        store.set_cookie("buvid3", buvid3);
        if let Some(buvid4) = buvid4 {
            store.set_cookie("buvid4", buvid4);
        }
        Ok(())
    }
}
