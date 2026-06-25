//! cargo run --example record_comment
//!
//! Records comment module API responses.
//!   Read-only APIs (comment_list / comment_list_wbi) require no login
//!   Write APIs require BILI_WRITE=1 and cookies.json

use bili::BiliClient;

const FIXTURES: &str = "fixtures";
const BVID: &str = "BV1nDJg6mEM5";

fn main() {
    std::fs::create_dir_all(FIXTURES).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = load_client();

    let aid = rt
        .block_on(client.resolve_bvid(BVID))
        .expect("resolve_bvid")
        .0;

    record(
        &rt,
        "comment_list",
        client.comment_list(1, aid, None, None, None),
    );
    record(
        &rt,
        "comment_list_wbi",
        client.comment_list_wbi(1, aid, None, None),
    );

    let write = std::env::var("BILI_WRITE").is_ok_and(|v| v == "1");
    if write {
        // post a comment first (with timestamp to avoid duplicates), get rpid then operate
        let msg = format!(
            "test comment from bili-sdk @{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        let rpid = match rt.block_on(client.comment_add(1, aid, &msg, None, None, None)) {
            Ok(v) => {
                record_raw("comment_add", &v);
                v["rpid"].as_i64().unwrap_or(0)
            }
            Err(e) => {
                let err = serde_json::json!({"error": e.to_string()});
                record_raw("comment_add", &err);
                eprintln!("ERR comment_add: {e}");
                0
            }
        };

        if rpid > 0 {
            record_unit(&rt, "comment_like", async {
                client
                    .comment_like(1, aid, rpid, Some(1))
                    .await
                    .map(|_| serde_json::json!({"success": true}))
            });
            record_unit(&rt, "comment_hate", async {
                client
                    .comment_hate(1, aid, rpid, Some(1))
                    .await
                    .map(|_| serde_json::json!({"success": true}))
            });
            record_unit(&rt, "comment_top", async {
                client
                    .comment_top(1, aid, rpid, Some(1))
                    .await
                    .map(|_| serde_json::json!({"success": true}))
            });
            record_unit(&rt, "comment_report", async {
                client
                    .comment_report(1, aid, rpid, 1, None)
                    .await
                    .map(|_| serde_json::json!({"success": true}))
            });
            record_unit(&rt, "comment_delete", async {
                client
                    .comment_delete(1, aid, rpid)
                    .await
                    .map(|_| serde_json::json!({"success": true}))
            });
        } else {
            eprintln!("SKIP comment ops: failed to create comment");
        }
    } else {
        eprintln!("SKIP write APIs: set BILI_WRITE=1");
    }
}

fn load_client() -> BiliClient {
    let c = BiliClient::new().expect("BiliClient::new()");
    if let Ok(json) = std::fs::read_to_string("cookies.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(c.import_cookies(
                v["sessdata"].as_str().map(String::from),
                v["bili_jct"].as_str().map(String::from),
                v["buvid3"].as_str().map(String::from),
                v["dedeuserid"].as_str().map(String::from),
            ));
        }
    }
    c
}

fn record(
    rt: &tokio::runtime::Runtime,
    name: &str,
    fut: impl std::future::Future<Output = Result<serde_json::Value, bili::BiliError>>,
) {
    let path = format!("{FIXTURES}/{name}.json");
    match rt.block_on(fut) {
        Ok(v) => {
            std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
            eprintln!("OK  {name}");
        }
        Err(e) => {
            let err = serde_json::json!({"error": e.to_string()});
            std::fs::write(&path, serde_json::to_string_pretty(&err).unwrap()).unwrap();
            eprintln!("ERR {name}: {e}");
        }
    }
}

fn record_raw(name: &str, value: &serde_json::Value) {
    let path = format!("{FIXTURES}/{name}.json");
    std::fs::write(&path, serde_json::to_string_pretty(value).unwrap()).unwrap();
    eprintln!("OK  {name}");
}

fn record_unit(
    rt: &tokio::runtime::Runtime,
    name: &str,
    fut: impl std::future::Future<Output = Result<serde_json::Value, bili::BiliError>>,
) {
    record(rt, name, fut)
}
