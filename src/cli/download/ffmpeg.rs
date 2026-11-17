//! The external `ffmpeg` dependency: presence check, remux, and temp cleanup.

use super::DownloadError;
use std::path::{Path, PathBuf};
use std::process::Output;

/// Fail fast if `ffmpeg` is missing, before downloading hundreds of megabytes.
///
/// # Errors
///
/// [`DownloadError::FfmpegMissing`] when the binary is absent, and
/// [`DownloadError::FfmpegUnusable`] when it exists but exits non-zero.
pub async fn ensure_ffmpeg() -> Result<(), DownloadError> {
    match tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(DownloadError::FfmpegUnusable(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(DownloadError::FfmpegMissing {
            purpose: "download DASH video",
        }),
        Err(e) => Err(e.into()),
    }
}

/// Run `ffmpeg` to remux the downloaded streams into `output`.
///
/// `-nostdin` keeps ffmpeg from competing for the terminal, which matters when
/// this runs under a pipe or in the background.
///
/// # Errors
///
/// [`DownloadError::FfmpegMissing`] when the binary disappeared after
/// [`ensure_ffmpeg`] succeeded, plus any spawn error.
pub async fn ffmpeg_merge(
    video: &Path,
    audio: Option<&PathBuf>,
    output: &Path,
) -> Result<Output, DownloadError> {
    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin").arg("-i").arg(video);
    if let Some(audio) = audio {
        cmd.arg("-i").arg(audio);
    }
    cmd.arg("-map").arg("0:v:0");
    if audio.is_some() {
        cmd.arg("-map").arg("1:a:0");
    }
    cmd.arg("-c").arg("copy").arg(output);

    match cmd.output().await {
        Ok(status) => Ok(status),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(DownloadError::FfmpegMissing {
            purpose: "merge DASH video/audio streams",
        }),
        Err(e) => Err(e.into()),
    }
}

/// Best-effort removal of intermediate files; failures are ignored on purpose.
pub async fn cleanup_files(paths: &[&Path]) {
    for path in paths {
        let _ = tokio::fs::remove_file(path).await;
    }
}
