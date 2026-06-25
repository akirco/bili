//! cargo run --example record_action
//!
//! Records action module API responses.
//!   Read-only APIs (favorite_list / has_like / has_coin / has_favorite) — login required
//!   Write APIs (like_video / coin_video / triple / favorite_add / summary_set) — requires BILI_WRITE=1

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

    let my_mid = rt.block_on(client.get_current_uid()).unwrap_or(2);

    // read-only (login required: has_like / has_coin / has_favorite)
    record(&rt, "favorite_list", client.favorite_list(my_mid));
    record(&rt, "has_like", client.has_like(aid));
    record(&rt, "has_coin", client.has_coin(aid));
    record(&rt, "has_favorite", client.has_favorite(aid));

    // write APIs (BILI_WRITE=1)
    let write = std::env::var("BILI_WRITE").is_ok_and(|v| v == "1");
    if write {
        // lookup own favorites, use first folder id
        let folder_id = rt
            .block_on(client.favorite_list(my_mid))
            .ok()
            .and_then(|v| v["data"]["list"][0]["id"].as_i64())
            .unwrap_or(0);

        record(&rt, "like_video", client.like_video(aid, true));
        record(
            &rt,
            "coin_video",
            client.coin_video(aid, Some(1), Some(false)),
        );
        record(&rt, "triple", client.triple(aid));
        if folder_id > 0 {
            record(&rt, "favorite_add", client.favorite_add(aid, folder_id));
            record(
                &rt,
                "fav_resource_deal",
                client.fav_resource_deal(aid, 2, Some(&folder_id.to_string()), None),
            );
        } else {
            eprintln!("SKIP favorite_add/fav_resource_deal: no folder found");
        }
    } else {
        eprintln!("SKIP write APIs: set BILI_WRITE=1");
    }
}

fn load_client() -> BiliClient {
    let c = BiliClient::new().expect("BiliClient::new()");
    if let Ok(json) = std::fs::read_to_string("cookies.json")
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&json)
    {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(c.import_cookies(
            v["sessdata"].as_str().map(String::from),
            v["bili_jct"].as_str().map(String::from),
            v["buvid3"].as_str().map(String::from),
            v["dedeuserid"].as_str().map(String::from),
        ));
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
