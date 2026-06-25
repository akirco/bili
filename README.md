# bili

- [ ] 架构重写
- [ ] 数据类型化

## 使用

```toml
[dependencies]
bili = "0.1.0"
tokio = { version = "*", features = ["full"] }
```

```rust
use bili::BiliClient;

#[tokio::main]
async fn main() {
    let client = BiliClient::new().expect("BiliClient::new()");

    // 获取视频信息
    let info = client.video_info(Some(12345), None).await.unwrap();
    println!("{:?}", info);

    // 搜索
    let results = client.search("rust", None).await.unwrap();
    println!("{:?}", results);
}
```

## 登录认证

部分接口需要登录。支持扫码登录：

```rust
use bili::BiliClient;

#[tokio::main]
async fn main() {
    let client = BiliClient::new().unwrap();

    // Step 1: 申请二维码
    let resp = client.login_qrcode_generate().await.unwrap();
    let url = resp["data"]["url"].as_str().unwrap();
    let key = resp["data"]["qrcode_key"].as_str().unwrap();
    println!("请用 Bilibili APP 扫描: {url}");

    // Step 2: 轮询扫码状态
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let poll = client.login_qrcode_poll(key).await.unwrap();
        if poll["data"]["code"].as_i64() == Some(0) {
            break; // 登录成功
        }
    }

    // Step 3: 导出 Cookie
    let (sessdata, bili_jct, buvid3, dedeuserid) = client.export_cookies().await;
    // 保存到 cookies.json ...
}
```

将 Cookie 保存到 `cookies.json`，后续启动时加载：

```rust
let client = BiliClient::new().unwrap();
if let Ok(json) = std::fs::read_to_string("cookies.json") {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
        client.import_cookies(
            v["sessdata"].as_str().map(String::from),
            v["bili_jct"].as_str().map(String::from),
            v["buvid3"].as_str().map(String::from),
            v["dedeuserid"].as_str().map(String::from),
        ).await;
    }
}
```

## 运行示例

先运行登录示例生成 `cookies.json`：

```sh
cargo run --example login
```

然后运行录制示例（部分需要 `cookies.json` 或 `BILI_WRITE=1`）：

```sh
# 无需登录
cargo run --example record_video
cargo run --example record_user
cargo run --example record_search
cargo run --example record_danmaku
cargo run --example record_audio

# 需要 cookies.json
cargo run --example record_action
cargo run --example record_comment
cargo run --example record_fav
cargo run --example record_history
```

## 项目结构

```
src/
├── lib.rs          # 入口，导出 BiliClient / BiliError
├── client.rs       # HTTP 客户端
├── credentials.rs  # Cookie 管理
├── error.rs        # 错误类型
├── wbi.rs          # WBI 签名
├── login.rs        # 扫码登录
├── audio.rs        # 音频流地址
├── coin.rs         # 投币
├── comment.rs      # 评论
├── danmaku.rs      # 弹幕
├── fav.rs          # 收藏夹
├── history.rs      # 历史记录 / 稍后再看
├── like.rs         # 点赞
├── search.rs       # 搜索 / 热搜
├── subtitle.rs     # AI 字幕
├── summary.rs      # AI 总结
├── triple.rs       # 一键三连
├── user.rs         # 用户信息
└── video.rs        # 视频信息
```

## License

MIT

## 参考

[BAC](https://github.com/pskdje/bilibili-API-collect/)
