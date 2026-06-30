//! cargo run --example record_history
//!
//! Records history module API responses.
//!   Login required; first run `cargo run --example login` to generate cookies.json

use bili::BiliClient;

const FIXTURES: &str = "fixtures";
const BVID: &str = "BV1nDJg6mEM5";

fn main() {
    std::fs::create_dir_all(FIXTURES).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = load_client();

    let _aid = rt
        .block_on(client.resolve_bvid(BVID))
        .expect("resolve_bvid")
        .0;

    record(&rt, "history_cursor", client.history().list(None, None));
}

fn load_client() -> BiliClient {
    let c = BiliClient::new().expect("BiliClient::new()");
    let json = std::fs::read_to_string("cookies.json").expect("先运行 cargo run --example login");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(c.import_cookies(
        v["sessdata"].as_str().map(String::from),
        v["bili_jct"].as_str().map(String::from),
        v["buvid3"].as_str().map(String::from),
        v["dedeuserid"].as_str().map(String::from),
    ));
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