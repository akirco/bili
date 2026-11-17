//! Downloading a video's DASH/`durl` streams to local files.
//!
//! This is the reusable half of `bili download`: it knows about play URLs,
//! resumable transfers, filename sanitisation and the `ffmpeg` remux, but
//! nothing about command-line arguments.

mod fetch;
mod ffmpeg;
mod filename;

pub use fetch::{download_one, download_stream, parse_content_range};
pub use ffmpeg::{cleanup_files, ensure_ffmpeg, ffmpeg_merge};
pub use filename::{MAX_FILENAME_BYTES, sanitize_filename};

use crate::BiliClient;
use crate::video::{DashVideoItem, VideoPage};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Anything that can stop a download.
#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    /// The API call that feeds the download failed.
    #[error(transparent)]
    Bili(#[from] crate::BiliError),
    /// The transfer itself failed.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// Writing the local file failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// `view` returned no `data` payload for the requested video.
    #[error("no video info")]
    MissingVideoInfo,
    /// `playurl` returned no `data` payload.
    #[error("no play url")]
    MissingPlayUrl,
    /// No stream of the requested kind was present in the response.
    #[error("no {0} stream")]
    MissingStream(&'static str),
    /// `--page` named a part the video does not have.
    #[error("no such page: P{0}")]
    NoSuchPage(i64),
    /// `--audio-only` was requested but the response has no DASH audio.
    #[error("this video has no DASH audio stream; cannot use --audio-only")]
    AudioOnlyUnsupported,
    /// Neither `dash` nor `durl` was usable.
    #[error("no playable url found")]
    NoPlayableUrl,
    /// `ffmpeg` is not installed.
    #[error("ffmpeg not found; install ffmpeg to {purpose}")]
    FfmpegMissing {
        /// What ffmpeg was needed for, e.g. `merge DASH video/audio streams`.
        purpose: &'static str,
    },
    /// `ffmpeg` is installed but exited non-zero for `-version`.
    #[error("ffmpeg is not usable: {0}")]
    FfmpegUnusable(String),
    /// The remux itself failed.
    #[error("ffmpeg failed: {0}")]
    FfmpegFailed(String),
    /// A partial file is already complete, or was truncated by an earlier run.
    #[error("partial file is already complete or corrupt: {}; delete it and retry", path.display())]
    StalePartialFile {
        /// The offending partial file.
        path: PathBuf,
    },
    /// The server resumed at a different offset than the local file size.
    #[error("server returned an unexpected range for {}; delete it and retry", path.display())]
    UnexpectedRange {
        /// The offending partial file.
        path: PathBuf,
    },
    /// Fewer bytes arrived than the server announced.
    #[error("incomplete download: {actual}/{expected} bytes (rerun to resume)")]
    Incomplete {
        /// Bytes written to disk.
        actual: u64,
        /// Bytes the server said to expect.
        expected: u64,
    },
}

/// Which part and quality to download.
#[derive(Debug, Clone, Copy, Default)]
pub struct DownloadOptions {
    /// Only this part number; `None` downloads every part.
    pub page: Option<i64>,
    /// Requested quality id (`qn`); `None` picks the highest bitrate.
    pub qn: Option<i64>,
    /// Request 4K streams (`fnval` bit 128).
    pub fourk: bool,
    /// Download the audio stream only, as `.m4a`.
    pub audio_only: bool,
}

/// Resolve `(aid, cid)` from a `bvid`, selecting a part when `page` is given.
///
/// # Errors
///
/// [`DownloadError::NoSuchPage`] when `page` is not part of the video, plus any
/// error from [`VideoClient::resolve_ids`].
pub async fn resolve_for_page(
    client: &BiliClient,
    bvid: &str,
    page: Option<i64>,
) -> Result<(i64, i64), DownloadError> {
    let ids = client.video().resolve_ids(bvid).await?;
    let cid = match page {
        Some(p) => ids.cid_of_page(p).ok_or(DownloadError::NoSuchPage(p))?,
        None => ids.cid,
    };
    Ok((ids.aid, cid))
}

/// Pick the requested quality, or the highest-bandwidth stream.
#[must_use]
pub fn select_video(videos: &[DashVideoItem], qn: Option<i64>) -> Option<&DashVideoItem> {
    if let Some(qn) = qn
        && let Some(video) = videos
            .iter()
            .filter(|video| video.id == qn)
            .max_by_key(|video| video.bandwidth)
    {
        return Some(video);
    }
    videos.iter().max_by_key(|video| video.bandwidth)
}

/// Select one part (or all parts) of a video.
///
/// # Errors
///
/// [`DownloadError::NoSuchPage`] when `page` is not part of the video.
pub fn select_pages(
    pages: &[VideoPage],
    page: Option<i64>,
) -> Result<Vec<&VideoPage>, DownloadError> {
    match page {
        Some(page) => {
            let selected: Vec<_> = pages.iter().filter(|part| part.page == page).collect();
            if selected.is_empty() {
                Err(DownloadError::NoSuchPage(page))
            } else {
                Ok(selected)
            }
        }
        None => Ok(pages.iter().collect()),
    }
}

/// Download `bvid` into `dir` and return a summary `{"status","title","files"}`.
///
/// Audio and video arrive as separate DASH streams and are remuxed with
/// `ffmpeg`; partial files survive failures so a rerun resumes them.
///
/// # Errors
///
/// Fails on any missing stream, a missing `ffmpeg`, a failed remux, or a
/// transfer error. Partial files are kept whenever a transfer fails, so the
/// caller can simply retry.
#[allow(clippy::too_many_lines)] // sequential download pipeline with explicit error cleanup
pub async fn handle_download(
    bvid: &str,
    dir: &str,
    options: DownloadOptions,
    client: &BiliClient,
) -> Result<serde_json::Value, DownloadError> {
    let info_resp = client.video().info(None, Some(bvid)).await?;
    let info = info_resp.data.ok_or(DownloadError::MissingVideoInfo)?;
    let title = &info.title;
    let total_pages = info.pages.len();
    let pages = select_pages(&info.pages, options.page)?;

    eprintln!("Downloading: {title} ({bvid}, {total_pages}P)");

    let dir = Path::new(dir);
    tokio::fs::create_dir_all(dir).await?;
    let mut downloaded = Vec::new();
    let mut ffmpeg_checked = false;

    for page in pages {
        let cid = page.cid;
        let part_name = if total_pages > 1 {
            sanitize_filename(&format!("{} - P{} {}", title, page.page, page.part))
        } else {
            sanitize_filename(title)
        };

        eprintln!("\n  [P{}/{}] {}", page.page, total_pages, part_name);

        let fnval = if options.fourk { 16 | 128 } else { 16 };
        let url_resp = client
            .video()
            .playurl(
                None,
                Some(bvid),
                cid,
                options.qn,
                Some(fnval),
                options.fourk.then_some(1),
            )
            .await?;
        let url_data = url_resp.data.ok_or(DownloadError::MissingPlayUrl)?;

        if let Some(dash) = &url_data.dash {
            let audio = dash.audio.iter().max_by_key(|a| a.bandwidth);

            if options.audio_only {
                let audio = audio.ok_or(DownloadError::MissingStream("audio"))?;
                let audio_tmp = dir.join(format!("{part_name}.audio.m4s"));
                let output = dir.join(format!("{part_name}.m4a"));
                eprintln!("    Audio: {} bps ({})", audio.bandwidth, audio.codecs);
                eprintln!("    Downloading audio stream...");
                if let Err(e) =
                    download_stream(client, &audio.base_url, &audio.backup_url, &audio_tmp).await
                {
                    eprintln!("    Partial file kept for resume: {}", audio_tmp.display());
                    return Err(e);
                }
                tokio::fs::rename(&audio_tmp, &output).await?;
                eprintln!("    Saved: {}", output.display());
                downloaded.push(output.display().to_string());
                continue;
            }

            let video = select_video(&dash.video, options.qn)
                .ok_or(DownloadError::MissingStream("video"))?;
            if let Some(qn) = options.qn
                && video.id != qn
            {
                eprintln!(
                    "    Requested qn {qn} is not available; using qn {} ({}p)",
                    video.id, video.height
                );
            }

            let video_tmp = dir.join(format!("{part_name}.video.m4s"));
            let audio_tmp = dir.join(format!("{part_name}.audio.m4s"));
            let output = dir.join(format!("{part_name}.mp4"));

            if !ffmpeg_checked {
                ensure_ffmpeg().await?;
                ffmpeg_checked = true;
            }

            eprintln!("    Video: {}p ({})", video.height, video.codecs);
            eprintln!("    Downloading video + audio streams...");
            let (video_res, audio_res) = tokio::join!(
                download_stream(client, &video.base_url, &video.backup_url, &video_tmp),
                async {
                    match audio {
                        Some(audio) => {
                            download_stream(client, &audio.base_url, &audio.backup_url, &audio_tmp)
                                .await
                        }
                        None => Ok(()),
                    }
                },
            );
            if let Err(e) = video_res {
                eprintln!("    Partial files kept for resume: {}", video_tmp.display());
                return Err(e);
            }
            if let Err(e) = audio_res {
                eprintln!(
                    "    Partial files kept for resume: {} / {}",
                    video_tmp.display(),
                    audio_tmp.display()
                );
                return Err(e);
            }

            eprintln!("    Merging with ffmpeg...");
            let status =
                ffmpeg_merge(&video_tmp, audio.is_some().then_some(&audio_tmp), &output).await?;
            if !status.status.success() {
                let stderr = String::from_utf8_lossy(&status.stderr);
                // A corrupt/HTML partial from an earlier failed download would
                // keep failing on every resume; drop the streams as well.
                cleanup_files(&[&video_tmp, &audio_tmp, &output]).await;
                return Err(DownloadError::FfmpegFailed(stderr.to_string()));
            }

            cleanup_files(&[&video_tmp, &audio_tmp]).await;

            eprintln!("    Saved: {}", output.display());
            downloaded.push(output.display().to_string());
        } else if options.audio_only {
            return Err(DownloadError::AudioOnlyUnsupported);
        } else if !url_data.durl.is_empty() {
            // The legacy API can split a video into multiple durl segments.
            // Download each one and append them into a single file.
            let output = dir.join(format!("{part_name}.mp4"));
            let mut out = tokio::fs::File::create(&output).await?;
            for (index, segment) in url_data.durl.iter().enumerate() {
                eprintln!(
                    "    Downloading segment {}/{}...",
                    index + 1,
                    url_data.durl.len()
                );
                let part = dir.join(format!("{part_name}.part{index}"));
                if let Err(e) =
                    download_stream(client, &segment.url, &segment.backup_url, &part).await
                {
                    drop(out);
                    cleanup_files(&[&part, &output]).await;
                    return Err(e);
                }
                let mut part_file = tokio::fs::File::open(&part).await?;
                tokio::io::copy(&mut part_file, &mut out).await?;
                drop(part_file);
                let _ = tokio::fs::remove_file(&part).await;
            }
            out.flush().await?;
            eprintln!("    Saved: {}", output.display());
            downloaded.push(output.display().to_string());
        } else {
            return Err(DownloadError::NoPlayableUrl);
        }
    }

    Ok(serde_json::json!({"status": "downloaded", "title": title, "files": downloaded}))
}
