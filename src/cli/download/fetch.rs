//! Resumable HTTP downloads into a local file.

use super::DownloadError;
use crate::BiliClient;
use reqwest::header::{CONTENT_RANGE, RANGE};
use std::path::Path;
use tokio::io::AsyncWriteExt;

/// Parse `Content-Range: bytes <start>-<end>/<total>`.
///
/// Returns `(start, total)`; `end` carries no information the caller needs, so
/// it is discarded. Returns `None` for `bytes */<total>` and for garbage.
#[must_use]
pub fn parse_content_range(value: &str) -> Option<(u64, u64)> {
    let rest = value.strip_prefix("bytes ")?;
    let (range, total) = rest.split_once('/')?;
    let (start, _end) = range.split_once('-')?;
    Some((start.parse().ok()?, total.parse().ok()?))
}

/// Download `url` to `path`, resuming an existing partial file when possible.
///
/// The CDN URLs are signed and can expire, so a failed download is resumed by
/// re-fetching a fresh play URL in the next run.
///
/// # Errors
///
/// Returns [`DownloadError::Io`] / [`DownloadError::Http`] for transport
/// failures, and the range/file-state variants when a resumed file no longer
/// matches what the server serves.
pub async fn download_one(
    client: &BiliClient,
    url: &str,
    path: &Path,
) -> Result<(), DownloadError> {
    let existing = tokio::fs::metadata(path)
        .await
        .map_or(0, |metadata| metadata.len());
    let mut request = client.download_http().get(url);
    if existing > 0 {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }

    // Use the download client (no total timeout) and fail on 403/404 instead
    // of writing an HTML error page into the .m4s file.
    let resp = request.send().await?;
    let status = resp.status();
    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE && existing > 0 {
        return Err(DownloadError::StalePartialFile {
            path: path.to_path_buf(),
        });
    }
    let resp = resp.error_for_status()?;
    let content_range = resp
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_content_range);
    let resuming = status == reqwest::StatusCode::PARTIAL_CONTENT && existing > 0;
    if resuming && content_range.is_none_or(|(start, _)| start != existing) {
        return Err(DownloadError::UnexpectedRange {
            path: path.to_path_buf(),
        });
    }
    let expected_total = if resuming {
        content_range.map(|(_, total)| total)
    } else {
        resp.content_length()
    };

    let mut resp = resp;
    let mut file = if resuming {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .await?
    } else {
        tokio::fs::File::create(path).await?
    };
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    if let Some(expected) = expected_total {
        let actual = tokio::fs::metadata(path).await?.len();
        if actual != expected {
            return Err(DownloadError::Incomplete { actual, expected });
        }
    }
    Ok(())
}

/// Download `url`, falling back to each of `backup_urls` in order.
///
/// # Errors
///
/// Returns the error from the primary URL when every backup also fails.
pub async fn download_stream(
    client: &BiliClient,
    url: &str,
    backup_urls: &[String],
    path: &Path,
) -> Result<(), DownloadError> {
    match download_one(client, url, path).await {
        Ok(()) => Ok(()),
        Err(first_error) => {
            for backup in backup_urls {
                if download_one(client, backup, path).await.is_ok() {
                    return Ok(());
                }
            }
            Err(first_error)
        }
    }
}
