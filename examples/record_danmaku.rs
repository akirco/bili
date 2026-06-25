//! cargo run --example record_danmaku
//!
//! Records danmaku module API responses. All APIs are accessible without login.

use bili::BiliClient;

const FIXTURES: &str = "fixtures";
const BVID: &str = "BV1nDJg6mEM5";

fn main() {
    std::fs::create_dir_all(FIXTURES).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = load_client();

    let (aid, cid) = rt
        .block_on(client.resolve_bvid(BVID))
        .expect("resolve_bvid");

    // get publish time from video info, infer danmaku month
    let info = rt
        .block_on(client.video_info(None, Some(BVID)))
        .expect("video_info");
    let pubdate = info["data"]["pubdate"].as_i64().unwrap_or(0);
    let month = chrono::DateTime::from_timestamp(pubdate, 0)
        .map(|t| t.format("%Y-%m").to_string())
        .unwrap_or_else(|| "2026-06".to_string());

    record(
        &rt,
        "danmaku_snapshot",
        client.get_danmaku_snapshot(&aid.to_string()),
    );
    record(
        &rt,
        "danmaku_history_dates",
        client.get_danmaku_history_dates(cid, &month),
    );
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
