//! Bilibili QR code login & cookie export
//!
//! Run: cargo run --example login
//! 1. A QR code is displayed in the terminal; scan it with the Bilibili app
//! 2. Cookies are auto-saved to cookies.json after confirmation
//! 3. The record_* examples will read this file

use std::time::Duration;

use bili::BiliClient;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = BiliClient::new().expect("BiliClient::new()");

        let resp = client
            .login()
            .qrcode()
            .await
            .expect("generate QR code failed");
        let url = resp["data"]["url"].as_str().expect("missing qrcode url");
        let qrcode_key = resp["data"]["qrcode_key"]
            .as_str()
            .expect("missing qrcode_key");

        println!("=== Bilibili 扫码登录 ===");
        println!("请用 Bilibili APP 扫描下方二维码：\n");

        if let Ok(code) = qrcode::QrCode::new(url.as_bytes()) {
            let image = code
                .render::<qrcode::render::unicode::Dense1x2>()
                .dark_color(qrcode::render::unicode::Dense1x2::Dark)
                .light_color(qrcode::render::unicode::Dense1x2::Light)
                .build();
            println!("{image}");
        }

        println!("\n或者手动打开链接：{url}");
        println!("二维码 key: {qrcode_key}");
        println!();

        loop {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let poll = client
                .login()
                .qrcode_status(qrcode_key)
                .await
                .expect("poll failed");
            let code = poll["data"]["code"].as_i64().unwrap_or(-1);
            match code {
                0 => {
                    println!("✅ 登录成功！");
                    break;
                }
                86090 => println!("⏳ 已扫码，等待确认..."),
                86101 => println!("⏳ 等待扫码..."),
                86038 => {
                    println!("❌ 二维码已过期，请重新运行");
                    return;
                }
                _ => {
                    let msg = poll["data"]["message"].as_str().unwrap_or("unknown");
                    println!("⚠️  状态 {code}: {msg}");
                }
            }
        }

        let creds = client.export_cookies().await;
        println!();
        println!(
            "  sessdata:    {:?}",
            creds.sessdata.as_ref().map(|s| &s[..8])
        );
        println!(
            "  bili_jct:    {:?}",
            creds.bili_jct.as_ref().map(|s| &s[..8])
        );
        println!("  dedeuserid:  {:?}", creds.dedeuserid);

        let json = serde_json::json!({
            "sessdata": creds.sessdata,
            "bili_jct": creds.bili_jct,
            "buvid3": creds.buvid3,
            "dedeuserid": creds.dedeuserid,
        });
        std::fs::write("cookies.json", serde_json::to_string_pretty(&json).unwrap())
            .expect("write cookies.json");
        println!("✅ Cookie 已保存到 cookies.json");
    });
}
