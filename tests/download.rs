//! Download helpers: filename sanitisation, part/quality selection, `Range`
//! handling, and resumable transfers against a mock CDN.

#![cfg(feature = "cli")]

use bili::BiliClient;
use bili::cli::download::{
    MAX_FILENAME_BYTES, download_one, parse_content_range, sanitize_filename, select_pages,
    select_video,
};
use bili::video::{DashVideoItem, VideoPage};
use std::path::PathBuf;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn pages() -> Vec<VideoPage> {
    vec![
        VideoPage {
            page: 1,
            ..VideoPage::default()
        },
        VideoPage {
            page: 2,
            ..VideoPage::default()
        },
    ]
}

fn temp_dir_path(tag: &str, extension: &str) -> PathBuf {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("bili_download_{tag}_{ts}.{extension}"))
}

#[test]
fn sanitize_filename_replaces_illegal_characters() {
    let name = sanitize_filename(r#"a/b:c*?"<>|"#);
    assert_eq!(name, "a_b_c______");
    assert!(
        !name
            .chars()
            .any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    );
}

#[test]
fn sanitize_filename_trims_and_falls_back() {
    assert_eq!(sanitize_filename("  ..  "), "video");
    assert_eq!(sanitize_filename("name... "), "name");
}

#[test]
fn sanitize_filename_truncates_long_titles() {
    let name = sanitize_filename(&"字".repeat(300));
    assert!(name.len() <= MAX_FILENAME_BYTES);
    assert_eq!(name.chars().count(), MAX_FILENAME_BYTES / 3);
}

#[test]
fn select_pages_supports_one_or_all_parts() {
    let parts = pages();
    assert_eq!(select_pages(&parts, None).unwrap().len(), 2);
    assert_eq!(select_pages(&parts, Some(2)).unwrap()[0].page, 2);
    assert!(select_pages(&parts, Some(99)).is_err());
}

#[test]
fn parses_content_range() {
    assert_eq!(parse_content_range("bytes 3-5/6"), Some((3, 6)));
    assert_eq!(parse_content_range("bytes */6"), None);
    assert_eq!(parse_content_range("garbage"), None);
}

#[test]
fn select_video_honors_requested_quality() {
    let videos = vec![
        DashVideoItem {
            id: 80,
            bandwidth: 1_000,
            ..DashVideoItem::default()
        },
        DashVideoItem {
            id: 120,
            bandwidth: 3_000,
            ..DashVideoItem::default()
        },
    ];
    assert_eq!(select_video(&videos, None).unwrap().id, 120);
    assert_eq!(select_video(&videos, Some(80)).unwrap().id, 80);
    // Unavailable quality falls back to the highest-bandwidth stream.
    assert_eq!(select_video(&videos, Some(999)).unwrap().id, 120);
}

#[tokio::test]
async fn download_resumes_a_partial_file() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/stream"))
        .respond_with(
            ResponseTemplate::new(206)
                .insert_header("content-range", "bytes 3-5/6")
                .set_body_string("def"),
        )
        .mount(&server)
        .await;

    let cookie_path = temp_dir_path("cookie", "json");
    let stream_path = temp_dir_path("stream", "m4s");
    std::fs::write(&stream_path, b"abc").unwrap();
    let client = BiliClient::builder()
        .cookie_path(cookie_path.clone())
        .build()
        .unwrap();

    download_one(&client, &format!("{}/stream", server.uri()), &stream_path)
        .await
        .expect("resume");
    assert_eq!(std::fs::read(&stream_path).unwrap(), b"abcdef");

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
    let _ = std::fs::remove_file(stream_path);
}

#[tokio::test]
async fn download_restarts_when_server_ignores_range() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/stream"))
        .respond_with(ResponseTemplate::new(200).set_body_string("xyz"))
        .mount(&server)
        .await;

    let cookie_path = temp_dir_path("cookie", "json");
    let stream_path = temp_dir_path("stream", "m4s");
    std::fs::write(&stream_path, b"abc").unwrap();
    let client = BiliClient::builder()
        .cookie_path(cookie_path.clone())
        .build()
        .unwrap();

    download_one(&client, &format!("{}/stream", server.uri()), &stream_path)
        .await
        .expect("restart");
    assert_eq!(std::fs::read(&stream_path).unwrap(), b"xyz");

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
    let _ = std::fs::remove_file(stream_path);
}
