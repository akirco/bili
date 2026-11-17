//! Request transport: query/form building, WBI signing, retries, and the
//! `get` / `post` entry points.

use super::{BiliClient, BiliResponse, FormData, Params, is_wbi_failure};
use crate::error::BiliError;
use crate::wbi::{WbiCache, signed_params_with_mixin};
use reqwest::RequestBuilder;
use reqwest::header::COOKIE;
use serde::de::DeserializeOwned;
use std::time::Duration;

impl BiliClient {
    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    async fn try_wbi_cache(&self) -> Option<String> {
        let cache = self.inner.wbi_cache.read().await;
        cache
            .as_ref()
            .filter(|c| c.is_valid())
            .map(|c| c.mixin_key.clone())
    }

    /// Return the cached WBI `mixin_key`, fetching and caching it if needed.
    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn get_wbi_keys(&self) -> Result<String, BiliError> {
        if let Some(mixin_key) = self.try_wbi_cache().await {
            return Ok(mixin_key);
        }

        // Fetch without holding the lock so concurrent WBI requests are not
        // serialized behind this network call.
        let value = self
            .get_raw(
                "https://api.bilibili.com/x/web-interface/nav",
                Params::new(),
            )
            .await?;

        let img_url = value["data"]["wbi_img"]["img_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.img_url missing".into()))?;
        let sub_url = value["data"]["wbi_img"]["sub_url"]
            .as_str()
            .ok_or_else(|| BiliError::Parse("wbi_img.sub_url missing".into()))?;

        let img_key = img_url
            .rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid img_url format".into()))?;

        let sub_key = sub_url
            .rsplit('/')
            .next()
            .and_then(|s| s.split('.').next())
            .ok_or_else(|| BiliError::Parse("invalid sub_url format".into()))?;

        let cache = WbiCache::with_ttl(img_key, sub_key, self.inner.wbi_ttl);
        let mixin_key = cache.mixin_key.clone();

        // Double-check: another task may have refreshed the cache while we
        // were fetching. Prefer whichever value is already there.
        let mut guard = self.inner.wbi_cache.write().await;
        if let Some(ref existing) = *guard
            && existing.is_valid()
        {
            return Ok(existing.mixin_key.clone());
        }
        *guard = Some(cache);

        Ok(mixin_key)
    }

    fn with_cookie(
        &self,
        req: RequestBuilder,
        url: &reqwest::Url,
    ) -> Result<RequestBuilder, BiliError> {
        let cookie_header = self
            .inner
            .cookie_store
            .lock()
            .map_err(|_| BiliError::CookieBuild("cookie store lock poisoned".into()))?
            .build_cookie_header_for(url);
        if let Some(cookie) = cookie_header {
            Ok(req.header(COOKIE, cookie))
        } else {
            Ok(req)
        }
    }

    /// Send a request, optionally retrying transient failures.
    ///
    /// `retryable` must be `false` for non-idempotent requests (POST): a 5xx or
    /// timeout may mean the server already applied the action, and retrying
    /// would duplicate comments/danmaku/coins.
    async fn send_retry<F>(
        &self,
        url: &str,
        build: F,
        retryable: bool,
    ) -> Result<reqwest::Response, BiliError>
    where
        F: Fn() -> RequestBuilder,
    {
        let parsed = reqwest::Url::parse(url).map_err(|e| BiliError::Parse(e.to_string()))?;
        let max_attempts = if retryable { 3 } else { 1 };
        let mut delay = Duration::from_millis(100);
        for attempt in 0..max_attempts {
            let req = self.with_cookie(build(), &parsed)?;
            match req.send().await {
                Ok(resp) => {
                    // Auto-export Set-Cookie from the response URL's domain.
                    {
                        let headers = resp.headers().clone();
                        let response_url = resp.url().clone();
                        if let Ok(mut store) = self.inner.cookie_store.lock() {
                            store.update_from_response(&response_url, &headers);
                        }
                    }
                    let status = resp.status();
                    let transient = status.is_server_error()
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS;
                    if transient && retryable && attempt + 1 < max_attempts {
                        tokio::time::sleep(delay).await;
                        delay *= 2;
                        continue;
                    }
                    // A cross-site redirect is stopped by the redirect policy;
                    // surface it instead of following without cookies or, worse,
                    // forwarding the Cookie header to an unrelated host.
                    if status.is_redirection() {
                        return Err(BiliError::Parse(format!(
                            "cross-site redirect blocked for cookie safety: {}",
                            resp.url()
                        )));
                    }
                    // Surface 4xx/5xx (e.g. 412 rate limiting, which returns
                    // HTML) as an HTTP error instead of a confusing JSON error.
                    if status == reqwest::StatusCode::PRECONDITION_FAILED
                        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
                    {
                        return Err(BiliError::RateLimited);
                    }
                    if !status.is_success() {
                        return Err(resp.error_for_status().unwrap_err().into());
                    }
                    return Ok(resp);
                }
                Err(e)
                    if retryable
                        && attempt + 1 < max_attempts
                        && (e.is_timeout() || e.is_connect()) =>
                {
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
                Err(e) => return Err(e.into()),
            }
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    async fn invalidate_wbi_cache(&self) {
        *self.inner.wbi_cache.write().await = None;
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_get_raw(
        &self,
        url: &str,
        params: Params,
    ) -> Result<serde_json::Value, BiliError> {
        let mut mixin_key = self.get_wbi_keys().await?;
        for attempt in 0..2 {
            let signed = signed_params_with_mixin(&params, &mixin_key);
            let raw = self.get_raw(url, signed).await?;
            if attempt == 0 && is_wbi_failure(&raw) {
                // Keys rotate daily; a stale cached key makes Bilibili return
                // -352/-403 or a `v_voucher`. Refresh once and retry.
                self.invalidate_wbi_cache().await;
                mixin_key = self.get_wbi_keys().await?;
                continue;
            }
            return Ok(raw);
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let raw = self.wbi_get_raw(url, params).await?;
        if is_wbi_failure(&raw) {
            return Err(BiliError::RateLimited);
        }
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
    ) -> Result<BiliResponse<T>, BiliError> {
        let resp = self
            .send_retry(
                url,
                || self.inner.http.get(url).query(&params.as_query()),
                true,
            )
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn get_raw(&self, url: &str, params: Params) -> Result<serde_json::Value, BiliError> {
        let resp = self
            .send_retry(
                url,
                || self.inner.http.get(url).query(&params.as_query()),
                true,
            )
            .await?;
        Ok(resp.json().await?)
    }

    /// Fetch a URL and return the body as text.
    ///
    /// Used by endpoints that are gated behind a token scraped out of an HTML
    /// page rather than served by the JSON API (see `live::LiveClient`).
    pub(crate) async fn get_text(&self, url: &str, params: Params) -> Result<String, BiliError> {
        let resp = self
            .send_retry(
                url,
                || {
                    self.inner
                        .http
                        .get(url)
                        .query(&params.as_query())
                        .header(reqwest::header::ACCEPT, "text/html,application/xhtml+xml")
                },
                true,
            )
            .await?;
        Ok(resp.text().await?)
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_post_raw(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<serde_json::Value, BiliError> {
        let mut mixin_key = self.get_wbi_keys().await?;
        for attempt in 0..2 {
            let signed = signed_params_with_mixin(&params, &mixin_key);
            let resp = self
                .send_retry(
                    url,
                    || {
                        self.inner
                            .http
                            .post(url)
                            .query(&signed.as_query())
                            .form(form)
                    },
                    false,
                )
                .await?;
            let raw: serde_json::Value = resp.json().await?;
            if attempt == 0 && is_wbi_failure(&raw) {
                self.invalidate_wbi_cache().await;
                mixin_key = self.get_wbi_keys().await?;
                continue;
            }
            return Ok(raw);
        }
        unreachable!()
    }

    #[allow(dead_code)] // only used by feature-gated WBI endpoints
    pub(crate) async fn wbi_post<T: DeserializeOwned>(
        &self,
        url: &str,
        params: Params,
        form: &FormData,
    ) -> Result<BiliResponse<T>, BiliError> {
        let raw = self.wbi_post_raw(url, params, form).await?;
        if is_wbi_failure(&raw) {
            return Err(BiliError::RateLimited);
        }
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, BiliError> {
        let resp = self
            .send_retry(url, || self.inner.http.get(url), true)
            .await?;
        Ok(resp.bytes().await?.to_vec())
    }

    #[must_use]
    pub fn http(&self) -> &reqwest::Client {
        &self.inner.http
    }

    /// HTTP client for large file downloads.
    ///
    /// Unlike [`Self::http`] it has no total-request timeout (only a 60s read
    /// timeout), so slow multi-hundred-MB downloads are not cut off.
    #[must_use]
    pub fn download_http(&self) -> &reqwest::Client {
        &self.inner.download_http
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        url: &str,
        form: &FormData,
    ) -> Result<BiliResponse<T>, BiliError> {
        let resp = self
            .send_retry(url, || self.inner.http.post(url).form(form), false)
            .await?;
        Ok(resp.json().await?)
    }
}
