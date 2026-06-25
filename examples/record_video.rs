//! cargo run --example record_video
//!
//! Records video module API responses. All APIs are accessible without login.

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

    record(&rt, "video_info", client.video_info(Some(aid), None));
    record(
        &rt,
        "video_info_by_bvid",
        client.video_info(None, Some(BVID)),
    );
    record(
        &rt,
        "video_playurl",
        client.video_playurl(Some(aid), None, cid, None, None, None),
    );
    record(&rt, "popular_videos", client.popular_videos(None, None));
    record(&rt, "video_related", client.video_related(Some(aid), None));
    record(&rt, "hot_videos", client.hot_videos(None, None, None));
    record(&rt, "rank_videos", client.rank_videos("0"));
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
