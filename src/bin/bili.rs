use bili::BiliClient;
use clap::Parser;
use clap_complete::generate;
use qrcode::QrCode;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/bili";
const CONFIG_FILE: &str = "cookies.json";

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(CONFIG_DIR).join(CONFIG_FILE)
}

#[derive(Parser)]
#[command(name = "bili", about = "Bilibili API CLI")]
struct Cli {
    // Actions
    #[arg(
        long,
        help = "Get video info [requires: --aid|--bvid]",
        help_heading = "Actions"
    )]
    info: bool,
    #[arg(long, help = "Popular videos", help_heading = "Actions")]
    popular: bool,
    #[arg(
        long,
        help = "Related recommendations [requires: --aid|--bvid]",
        help_heading = "Actions"
    )]
    related: bool,
    #[arg(long, help = "Homepage recommendations", help_heading = "Actions")]
    rcmd: bool,
    #[arg(long, help = "Ranking by region", help_heading = "Actions")]
    hot: bool,
    #[arg(
        long,
        help = "User profile [requires: --mid]",
        help_heading = "Actions"
    )]
    user: bool,
    #[arg(
        long,
        help = "User submitted videos [requires: --mid]",
        help_heading = "Actions"
    )]
    user_videos: bool,
    #[arg(long, help = "User stats [requires: --mid]", help_heading = "Actions")]
    user_stat: bool,
    #[arg(
        long,
        help = "Following list [requires: --mid]",
        help_heading = "Actions"
    )]
    followings: bool,
    #[arg(
        long,
        help = "Follow a user [requires: --fid]",
        help_heading = "Actions"
    )]
    follow: bool,
    #[arg(
        long,
        help = "Unfollow a user [requires: --fid]",
        help_heading = "Actions"
    )]
    unfollow: bool,
    #[arg(long, help = "Search (value=keyword)", help_heading = "Actions")]
    search: Option<String>,
    #[arg(long, help = "Hot search words", help_heading = "Actions")]
    hot_search: bool,
    #[arg(
        long,
        help = "Comment list [requires: --oid]",
        help_heading = "Actions"
    )]
    comments: bool,
    #[arg(
        long,
        help = "Post a comment [requires: --oid, value=message]",
        help_heading = "Actions"
    )]
    add_comment: Option<String>,
    #[arg(
        long,
        help = "Favorite folder list [requires: --mid]",
        help_heading = "Actions"
    )]
    fav_list: bool,
    #[arg(
        long,
        help = "Check if video is favorited [requires: --aid]",
        help_heading = "Actions"
    )]
    has_favorite: bool,
    #[arg(
        long,
        help = "Danmaku list [requires: --cid]",
        help_heading = "Actions"
    )]
    danmaku: bool,
    #[arg(
        long,
        help = "Send danmaku [requires: --oid, value=message]",
        help_heading = "Actions"
    )]
    send_danmaku: Option<String>,
    #[arg(long, help = "Watch history", help_heading = "Actions")]
    history: bool,
    #[arg(long, help = "Watch later list", help_heading = "Actions")]
    toview: bool,
    #[arg(
        long,
        help = "Add video to watch later [requires: --aid|--bvid]",
        help_heading = "Actions"
    )]
    toview_add: bool,
    #[arg(long, help = "Login via QR code", help_heading = "Actions")]
    login: bool,
    #[arg(
        long,
        help = "Like a video [requires: --aid]",
        help_heading = "Actions"
    )]
    like: bool,
    #[arg(
        long,
        help = "Coin a video [requires: --aid]",
        help_heading = "Actions"
    )]
    coin: bool,
    #[arg(
        long,
        help = "Triple like+coin+fav [requires: --aid]",
        help_heading = "Actions"
    )]
    triple: bool,
    #[arg(
        long,
        help = "Subtitle list [requires: --aid, --cid]",
        help_heading = "Actions"
    )]
    subtitle: bool,
    #[arg(
        long,
        help = "Subtitle summary [requires: --aid, --cid]",
        help_heading = "Actions"
    )]
    subtitle_summary: bool,
    #[arg(
        long,
        help = "Audio stream URL [requires: --sid]",
        help_heading = "Actions"
    )]
    audio: bool,
    #[arg(
        long,
        help = "Video play URL [requires: --aid|--bvid, --cid]",
        help_heading = "Actions"
    )]
    playurl: bool,

    // IDs
    #[arg(long, help = "Video avid", help_heading = "IDs")]
    aid: Option<i64>,
    #[arg(long, help = "Video bvid", help_heading = "IDs")]
    bvid: Option<String>,
    #[arg(long, help = "Video cid", help_heading = "IDs")]
    cid: Option<i64>,
    #[arg(long, help = "User mid", help_heading = "IDs")]
    mid: Option<i64>,
    #[arg(long, help = "Target user mid (follow/unfollow)", help_heading = "IDs")]
    fid: Option<i64>,
    #[arg(
        long,
        help = "Object id (comment oid / danmaku cid)",
        help_heading = "IDs"
    )]
    oid: Option<i64>,
    #[arg(long, help = "Audio sid", help_heading = "IDs")]
    sid: Option<i64>,

    // Options
    #[arg(
        long,
        help = "Type (comment: 1=video; search: video/bili_user/...)",
        help_heading = "Options"
    )]
    r#type: Option<String>,
    #[arg(
        long,
        help = "Sort (comment: 0=time,2=like,3=reply)",
        help_heading = "Options"
    )]
    sort: Option<i64>,
    #[arg(
        long,
        help = "Mode (danmaku: 1=normal,4=bottom,5=top)",
        help_heading = "Options"
    )]
    mode: Option<i32>,
    #[arg(long, help = "Coin count (1 or 2)", help_heading = "Options")]
    multiply: Option<i64>,
    #[arg(
        long,
        help = "Playback position in ms (danmaku)",
        help_heading = "Options"
    )]
    progress: Option<i64>,
    #[arg(long, help = "Region id (ranking)", help_heading = "Options")]
    rid: Option<String>,
    #[arg(long, help = "Page number", help_heading = "Options")]
    page: Option<i64>,
    #[arg(long, help = "Page number", help_heading = "Options")]
    pn: Option<i64>,
    #[arg(long, help = "Page size", help_heading = "Options")]
    ps: Option<i64>,
    #[arg(
        long,
        help = "Limit / max segments (danmaku)",
        help_heading = "Options"
    )]
    limit: Option<i64>,
    #[arg(long, help = "Quality (16=360P..127=8K)", help_heading = "Options")]
    qn: Option<i64>,
    #[arg(
        long,
        help = "Format (1=MP4,16=DASH,4095=all)",
        help_heading = "Options"
    )]
    fnval: Option<i64>,
    #[arg(long, help = "Allow 4K (0=no,1=yes)", help_heading = "Options")]
    fourk: Option<i64>,
    #[arg(
        long,
        help = "Segment index (subtitle summary)",
        help_heading = "Options"
    )]
    segment: Option<i64>,

    // Global
    #[arg(
        long,
        help = "Cookie string (SESSDATA=xx; bili_jct=xx)",
        help_heading = "Global"
    )]
    cookies: Option<String>,
    #[arg(long, help = "Import cookies from file", help_heading = "Global")]
    import_cookies: Option<String>,
    #[arg(long, help = "Export cookies to file", help_heading = "Global")]
    export_cookies: Option<String>,
    #[arg(long, help = "Output raw JSON (single line)", help_heading = "Global")]
    raw: bool,
    #[arg(long, help = "Output data field only", help_heading = "Global")]
    compact: bool,
    #[arg(
        long = "completions",
        value_name = "SHELL",
        help = "Generate shell completions (bash/zsh/fish/powershell/elvish)",
        help_heading = "Global"
    )]
    completions: Option<String>,
}

fn has_action(cli: &Cli) -> bool {
    cli.info
        || cli.popular
        || cli.related
        || cli.rcmd
        || cli.hot
        || cli.user
        || cli.user_videos
        || cli.user_stat
        || cli.followings
        || cli.follow
        || cli.unfollow
        || cli.search.is_some()
        || cli.hot_search
        || cli.comments
        || cli.add_comment.is_some()
        || cli.fav_list
        || cli.has_favorite
        || cli.danmaku
        || cli.send_danmaku.is_some()
        || cli.history
        || cli.toview
        || cli.login
        || cli.like
        || cli.coin
        || cli.triple
        || cli.subtitle
        || cli.subtitle_summary
        || cli.audio
        || cli.playurl
        || cli.toview_add
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle --completions early (doesn't need client)
    if let Some(ref shell) = cli.completions {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        let shell: clap_complete::Shell = shell.parse().unwrap_or_else(|_| {
            eprintln!("Unknown shell: {shell}. Use bash, zsh, fish, powershell, or elvish.");
            std::process::exit(1);
        });
        generate(shell, &mut cmd, "bili", &mut std::io::stdout());
        return;
    }

    if !has_action(&cli) {
        eprintln!("No action specified. Use --help for usage.");
        std::process::exit(1);
    }

    let client = BiliClient::new().expect("Failed to create client");

    // Auto-load cookies from config
    let cfg = config_path();
    if cfg.exists() {
        let _ = client.import_cookies(Some(cfg.to_str().unwrap())).await;
    }

    // Apply --cookies
    if let Some(ref cookies) = cli.cookies {
        let params: Vec<&str> = cookies.split(';').map(|s| s.trim()).collect();
        let url = format!("https://bilibili.com/?{}", params.join("&"));
        if let Err(e) = client.parse_cookies(&url).await {
            eprintln!("Failed to parse cookies: {}", e);
            std::process::exit(1);
        }
    }

    // Apply --import-cookies
    if let Some(ref path) = cli.import_cookies {
        if let Err(e) = client.import_cookies(Some(path)).await {
            eprintln!("Failed to import cookies: {}", e);
            std::process::exit(1);
        }
    }

    // Run action
    let result = match run(&cli, &client).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Export cookies
    if let Some(ref path) = cli.export_cookies {
        if let Err(e) = client.export_cookies(Some(path)).await {
            eprintln!("Failed to export cookies: {}", e);
            std::process::exit(1);
        }
    }

    print_output(&result, cli.raw, cli.compact);
}

async fn run(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if cli.info {
        return handle_info(cli, client).await;
    }
    if cli.playurl {
        return handle_playurl(cli, client).await;
    }
    if cli.popular {
        return handle_popular(cli, client).await;
    }
    if cli.related {
        return handle_related(cli, client).await;
    }
    if cli.rcmd {
        return handle_rcmd(cli, client).await;
    }
    if cli.hot {
        return handle_hot(cli, client).await;
    }
    if cli.user {
        return handle_user(cli, client).await;
    }
    if cli.user_videos {
        return handle_user_videos(cli, client).await;
    }
    if cli.user_stat {
        return handle_user_stat(cli, client).await;
    }
    if cli.followings {
        return handle_followings(cli, client).await;
    }
    if cli.follow {
        return handle_follow(cli, client).await;
    }
    if cli.unfollow {
        return handle_unfollow(cli, client).await;
    }
    if cli.search.is_some() {
        return handle_search(cli, client).await;
    }
    if cli.hot_search {
        return handle_hot_search(cli, client).await;
    }
    if cli.comments {
        return handle_comments(cli, client).await;
    }
    if cli.add_comment.is_some() {
        return handle_add_comment(cli, client).await;
    }
    if cli.fav_list {
        return handle_fav_list(cli, client).await;
    }
    if cli.has_favorite {
        return handle_has_favorite(cli, client).await;
    }
    if cli.danmaku {
        return handle_danmaku(cli, client).await;
    }
    if cli.send_danmaku.is_some() {
        return handle_send_danmaku(cli, client).await;
    }
    if cli.history {
        return handle_history(cli, client).await;
    }
    if cli.toview {
        return handle_toview(cli, client).await;
    }
    if cli.toview_add {
        return handle_toview_add(cli, client).await;
    }
    if cli.login {
        return handle_login(client).await;
    }
    if cli.like {
        return handle_like(cli, client).await;
    }
    if cli.coin {
        return handle_coin(cli, client).await;
    }
    if cli.triple {
        return handle_triple(cli, client).await;
    }
    if cli.subtitle {
        return handle_subtitle(cli, client).await;
    }
    if cli.subtitle_summary {
        return handle_subtitle_summary(cli, client).await;
    }
    if cli.audio {
        return handle_audio(cli, client).await;
    }
    unreachable!()
}

async fn handle_info(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if cli.aid.is_none() && cli.bvid.is_none() {
        return Err("requires --aid or --bvid".into());
    }
    let resp = client.video().info(cli.aid, cli.bvid.as_deref()).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_popular(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = client.video().popular(cli.pn, cli.ps).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_related(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if cli.aid.is_none() && cli.bvid.is_none() {
        return Err("requires --aid or --bvid".into());
    }
    let resp = client.video().related(cli.aid, cli.bvid.as_deref()).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_rcmd(
    _cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = client.video().rcmd(_cli.ps).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_hot(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = client
        .video()
        .hot(cli.pn, cli.ps, cli.rid.as_deref())
        .await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_playurl(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if cli.aid.is_none() && cli.bvid.is_none() {
        return Err("requires --aid or --bvid".into());
    }
    let cid = cli.cid.ok_or("missing --cid")?;
    let resp = client
        .video()
        .playurl(
            cli.aid,
            cli.bvid.as_deref(),
            cid,
            cli.qn,
            cli.fnval,
            cli.fourk,
        )
        .await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_user(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mid = cli.mid.ok_or("missing --mid")?;
    let resp = client.user().info(mid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_user_videos(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mid = cli.mid.ok_or("missing --mid")?;
    let resp = client.user().videos(mid, cli.pn, cli.ps).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_user_stat(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mid = cli.mid.ok_or("missing --mid")?;
    let resp = client.user().stat(mid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_followings(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mid = cli.mid.ok_or("missing --mid")?;
    let resp = client.user().followings(mid, cli.ps, cli.pn).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_follow(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let fid = cli.fid.ok_or("missing --fid")?;
    let resp = client.user().follow(fid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_unfollow(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let fid = cli.fid.ok_or("missing --fid")?;
    let resp = client.user().unfollow(fid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_search(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let keyword = cli.search.as_ref().unwrap();
    let value = if let Some(ref search_type) = cli.r#type {
        let api_type = match search_type.as_str() {
            "user" => "bili_user",
            other => other,
        };
        let items = client.search().query(keyword, cli.page, api_type).await?;
        serde_json::to_value(items)?
    } else {
        let result = client.search().all(keyword, cli.page).await?;
        serde_json::to_value(result)?
    };
    Ok(value)
}

async fn handle_hot_search(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let limit = cli.limit.unwrap_or(50);
    let result = client.search().hot(limit).await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_comments(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let oid = cli.oid.ok_or("missing --oid")?;
    let comment_type: i64 = cli.r#type.as_deref().unwrap_or("1").parse()?;
    let result = client
        .comment()
        .list(comment_type, oid, cli.sort, cli.pn, cli.ps)
        .await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_add_comment(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let oid = cli.oid.ok_or("missing --oid")?;
    let message = cli.add_comment.as_ref().unwrap();
    let comment_type: i64 = cli.r#type.as_deref().unwrap_or("1").parse()?;
    let resp = client
        .comment()
        .add(comment_type, oid, message, None, None, None)
        .await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_fav_list(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mid = cli.mid.ok_or("missing --mid")?;
    let result = client.fav().folder_list(mid).await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_has_favorite(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let resp = client.fav().has_favorite(aid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_danmaku(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let cid = cli.cid.ok_or("missing --cid")?;
    let max_seg = cli.limit.unwrap_or(10);
    let elems = client.danmaku().all_segments(cid, max_seg).await?;
    Ok(serde_json::to_value(elems)?)
}

async fn handle_send_danmaku(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let oid = cli.oid.ok_or("missing --oid")?;
    let msg = cli.send_danmaku.as_ref().unwrap();
    let mode = cli.mode.unwrap_or(1);
    let progress = cli.progress.unwrap_or(0);
    let resp = client
        .danmaku()
        .send(oid, msg, mode, progress, cli.aid, cli.bvid.as_deref())
        .await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_history(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let result = client.history().list(cli.pn, cli.ps).await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_toview(
    _cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let result = client.history().toview_list().await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_toview_add(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    if cli.aid.is_none() && cli.bvid.is_none() {
        return Err("requires --aid or --bvid".into());
    }
    let resp = client
        .history()
        .toview_add(cli.aid, cli.bvid.as_deref())
        .await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_login(
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = client.login().qrcode().await?;
    if resp.code != 0 {
        return Err(format!("Failed to generate QR code: {}", resp.message).into());
    }
    let data = resp.data.ok_or("No data in QR code response")?;

    let code = QrCode::new(data.url.as_bytes()).map_err(|e| format!("QR code error: {}", e))?;
    println!("Scan the QR code below to login:");
    println!(
        "{}",
        code.render::<qrcode::render::unicode::Dense1x2>()
            .quiet_zone(false)
            .build()
    );

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let status = client.login().qrcode_status(&data.qrcode_key).await?;
        if status.code != 0 {
            return Err(format!("Login failed: {}", status.message).into());
        }
        let s = status.data.ok_or("No data in status response")?;

        match s.code {
            0 => {
                if let Some(url) = s.url {
                    client.parse_cookies(&url).await?;
                }
                let path = config_path();
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                client.export_cookies(Some(path.to_str().unwrap())).await?;
                println!("Login successful! Cookies saved to {}", path.display());
                return Ok(serde_json::json!({"status": "success"}));
            }
            86090 => {
                eprintln!("QR code scanned, waiting for confirmation...");
            }
            86101 => {}
            86038 => {
                return Err("QR code expired, please try again".into());
            }
            other => {
                return Err(format!("Unexpected login status: {}", other).into());
            }
        }
    }
}

async fn handle_like(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let resp = client.action().like_video(aid, true).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_coin(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let resp = client.action().coin_video(aid, cli.multiply, None).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_triple(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let resp = client.action().triple(aid).await?;
    Ok(serde_json::to_value(resp)?)
}

async fn handle_subtitle(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let cid = cli.cid.ok_or("missing --cid")?;
    let result = client.subtitle().list(aid, cid).await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_subtitle_summary(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let aid = cli.aid.ok_or("missing --aid")?;
    let cid = cli.cid.ok_or("missing --cid")?;
    let result = client.subtitle().summary(aid, cid, cli.segment).await?;
    Ok(serde_json::to_value(result)?)
}

async fn handle_audio(
    cli: &Cli,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let sid = cli.sid.ok_or("missing --sid")?;
    let result = client.audio().stream_url(sid).await?;
    Ok(serde_json::to_value(result)?)
}

fn print_output(value: &serde_json::Value, raw: bool, compact: bool) {
    if raw {
        println!("{}", serde_json::to_string(value).unwrap());
    } else if compact {
        let output = value.get("data").unwrap_or(value);
        println!("{}", serde_json::to_string_pretty(output).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(value).unwrap());
    }
}
