//! cargo run --example record_fav
//!
//! Records fav module API responses.
//!   Read-only APIs require no login; write APIs need cookies.json

use bili::BiliClient;

const FIXTURES: &str = "fixtures";

fn main() {
    std::fs::create_dir_all(FIXTURES).unwrap();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let client = load_client();

    let my_mid = rt.block_on(client.get_current_uid()).unwrap_or(2);

    // use first folder ID as sample media_id
    let media_id = rt
        .block_on(client.favorite_folder_list_all(my_mid, None, None))
        .ok()
        .and_then(|v| v["data"]["list"][0]["id"].as_i64())
        .unwrap_or(0);

    // ── read-only ──
    if media_id > 0 {
        record(
            &rt,
            "favorite_folder_info",
            client.favorite_folder_info(media_id),
        );
        record(
            &rt,
            "favorite_resource_list",
            client.favorite_resource_list(media_id, None, None, None, None, None, None),
        );
        record(&rt, "favorite_resource_ids", async {
            client
                .favorite_resource_ids(media_id, None)
                .await
                .map(|v| serde_json::json!({"ids": v}))
        });
        record(
            &rt,
            "favorite_resource_infos",
            client.favorite_resource_infos(&format!("{media_id}:2"), None),
        );
    } else {
        eprintln!("SKIP media_id-dependent APIs: no folder found");
    }
    record(
        &rt,
        "favorite_folder_list_all",
        client.favorite_folder_list_all(my_mid, None, None),
    );
    record(
        &rt,
        "favorite_collected_list",
        client.favorite_collected_list(my_mid, None, None),
    );

    // ── write APIs (BILI_WRITE=1) ──
    let write = std::env::var("BILI_WRITE").is_ok_and(|v| v == "1");
    if write {
        let title = &format!("bili-sdk-{my_mid}");
        match rt.block_on(client.favorite_folder_add(title, None, Some(1), None)) {
            Ok(v) => {
                let path = format!("{FIXTURES}/favorite_folder_add.json");
                std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
                eprintln!("OK  favorite_folder_add");
                if let Some(mid) = v["data"]["media_id"].as_i64() {
                    let _ =
                        rt.block_on(client.favorite_folder_edit(mid, title, None, Some(1), None));
                    let path = format!("{FIXTURES}/favorite_folder_edit.json");
                    std::fs::write(
                        &path,
                        serde_json::to_string_pretty(&serde_json::json!({"success": true}))
                            .unwrap(),
                    )
                    .unwrap();
                    eprintln!("OK  favorite_folder_edit");

                    let _ = rt.block_on(client.favorite_folder_del(&mid.to_string()));
                    let path = format!("{FIXTURES}/favorite_folder_del.json");
                    std::fs::write(
                        &path,
                        serde_json::to_string_pretty(&serde_json::json!({"success": true}))
                            .unwrap(),
                    )
                    .unwrap();
                    eprintln!("OK  favorite_folder_del");
                }
            }
            Err(e) => {
                let err = serde_json::json!({"error": e.to_string()});
                let path = format!("{FIXTURES}/favorite_folder_add.json");
                std::fs::write(&path, serde_json::to_string_pretty(&err).unwrap()).unwrap();
                eprintln!("ERR favorite_folder_add: {e}");
            }
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
