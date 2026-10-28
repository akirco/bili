//! HTTP 层测试：用本地 mock server 验证重试策略、412 映射和 Set-Cookie 处理。

use bili::error::BiliError;
use bili::{BiliClient, FormBuilder, Params};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

fn temp_cookie_path() -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("bili_http_test_{ts}.json"))
}

fn test_client(path: &Path) -> BiliClient {
    BiliClient::builder()
        .cookie_path(path.to_path_buf())
        .build()
        .expect("client")
}

/// Responds with `failures` 500s, then a JSON success body.
struct Flaky {
    calls: Arc<AtomicUsize>,
    failures: usize,
    body: serde_json::Value,
}

impl Respond for Flaky {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        if call < self.failures {
            ResponseTemplate::new(500)
        } else {
            ResponseTemplate::new(200).set_body_json(self.body.clone())
        }
    }
}

/// Counts calls and always returns HTTP 500.
struct Always500 {
    calls: Arc<AtomicUsize>,
}

impl Respond for Always500 {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        self.calls.fetch_add(1, Ordering::SeqCst);
        ResponseTemplate::new(500)
    }
}

#[tokio::test]
async fn get_retries_transient_server_errors() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("GET"))
        .and(path("/history"))
        .respond_with(Flaky {
            calls: calls.clone(),
            failures: 1,
            body: serde_json::json!({
                "code": 0,
                "message": "OK",
                "ttl": 1,
                "data": {
                    "list": [],
                    "has_more": true,
                    "cursor": {"max": 1, "view_at": 2, "business": "archive", "ps": 20}
                }
            }),
        })
        .mount(&server)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let url = format!("{}/history", server.uri());
    let resp = client
        .get::<bili::types::HistoryResult>(&url, Params::new())
        .await
        .expect("retry should eventually succeed");

    assert!(resp.data.expect("data").has_more);
    assert_eq!(calls.load(Ordering::SeqCst), 2);

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
}

#[tokio::test]
async fn post_is_not_retried() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    Mock::given(method("POST"))
        .and(path("/comment"))
        .respond_with(Always500 {
            calls: calls.clone(),
        })
        .mount(&server)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let url = format!("{}/comment", server.uri());
    let form = FormBuilder::new().push("message", "hello").build();
    let err = client
        .post::<serde_json::Value>(&url, &form)
        .await
        .expect_err("500 should be an error");

    assert!(matches!(err, BiliError::Http(_)));
    assert_eq!(calls.load(Ordering::SeqCst), 1, "POST must not be retried");

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
}

#[tokio::test]
async fn http_412_maps_to_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/blocked"))
        .respond_with(ResponseTemplate::new(412).set_body_string("<html>blocked</html>"))
        .mount(&server)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let url = format!("{}/blocked", server.uri());
    let err = client
        .get::<serde_json::Value>(&url, Params::new())
        .await
        .expect_err("412 should fail");

    assert!(matches!(err, BiliError::RateLimited));

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
}

#[tokio::test]
async fn cross_site_redirect_is_blocked_to_protect_cookies() {
    let target = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/secret"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"code": 0, "data": null})),
        )
        .mount(&target)
        .await;

    let source = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/redirect"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("location", format!("{}/secret", target.uri())),
        )
        .mount(&source)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let err = client
        .get::<serde_json::Value>(&format!("{}/redirect", source.uri()), Params::new())
        .await
        .expect_err("cross-site redirect must be blocked");

    assert!(matches!(err, BiliError::Parse(ref message) if message.contains("redirect")));
    let received = target.received_requests().await.unwrap_or_default();
    assert!(
        received.is_empty(),
        "cookie-bearing request was forwarded to another host"
    );

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
}

#[tokio::test]
async fn same_origin_redirect_is_followed() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/a"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/b"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/b"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"code": 0, "message": "OK", "data": {"ok": true}}),
            ),
        )
        .mount(&server)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let resp = client
        .get::<serde_json::Value>(&format!("{}/a", server.uri()), Params::new())
        .await
        .expect("same-origin redirect should be followed");
    assert_eq!(resp.data.unwrap()["ok"], true);

    drop(client);
    let _ = std::fs::remove_file(cookie_path);
}

#[tokio::test]
async fn set_cookie_updates_cookie_store() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "SESSDATA=test-token; Path=/")
                .set_body_json(serde_json::json!({
                    "code": 0,
                    "message": "OK",
                    "ttl": 1,
                    "data": null
                })),
        )
        .mount(&server)
        .await;

    let cookie_path = temp_cookie_path();
    let client = test_client(&cookie_path);
    let url = format!("{}/login", server.uri());
    client
        .get::<serde_json::Value>(&url, Params::new())
        .await
        .expect("request");

    assert!(client.is_logged_in());

    drop(client);
    let saved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cookie_path).unwrap()).unwrap();
    assert_eq!(saved["cookies"]["SESSDATA"]["value"], "test-token");
    let _ = std::fs::remove_file(cookie_path);
}
