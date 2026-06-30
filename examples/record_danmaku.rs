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

    let info = rt
        .block_on(client.video().info(None, Some(BVID)))
        .expect("video_info");
    let pubdate = info["data"]["pubdate"].as_i64().unwrap_or(0);
    let month = epoch_to_month(pubdate);

    record(
        &rt,
        "danmaku_snapshot",
        client.danmaku().snapshot(&aid.to_string()),
    );
    record(
        &rt,
        "danmaku_history_dates",
        client.danmaku().history_dates(cid, &month),
    );
}

fn epoch_to_month(epoch: i64) -> String {
    if epoch <= 0 {
        return "2026-06".to_string();
    }
    let mut seconds = epoch as u64;
    let mut year = 1970;
    loop {
        let year_seconds = if is_leap_year(year) { 366 * 24 * 60 * 60 } else { 365 * 24 * 60 * 60 };
        if seconds < year_seconds {
            break;
        }
        seconds -= year_seconds;
        year += 1;
    }
    let mut month = 1;
    let months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    loop {
        let m = if month == 2 && is_leap_year(year) { 29 } else { months[month - 1] };
        let month_seconds = m as u64 * 24 * 60 * 60;
        if seconds < month_seconds {
            break;
        }
        seconds -= month_seconds;
        month += 1;
    }
    format!("{year}-{month:02}")
}

fn is_leap_year(year: i32) -> bool {
    if year % 4 != 0 {
        return false;
    }
    if year % 100 != 0 {
        return true;
    }
    year % 400 == 0
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