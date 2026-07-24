use bili::BiliClient;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use qrcode::QrCode;
use qrcode::types::EcLevel;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, ValueEnum)]
enum SearchType {
    Video,
    #[value(name = "bili_user")]
    BiliUser,
    MediaBangumi,
    MediaFt,
    Live,
    LiveRoom,
    LiveUser,
    Article,
    Topic,
    Photo,
}

#[derive(Debug, Clone, ValueEnum)]
enum CommentType {
    #[value(name = "1")]
    Video,
    #[value(name = "11")]
    ImageText,
    #[value(name = "17")]
    Dynamic,
}

#[derive(Debug, Clone, ValueEnum)]
enum CommentSort {
    #[value(name = "0")]
    Time,
    #[value(name = "2")]
    Like,
    #[value(name = "3")]
    Reply,
}

#[derive(Debug, Clone, ValueEnum)]
enum DanmakuMode {
    #[value(name = "1")]
    Normal,
    #[value(name = "4")]
    Bottom,
    #[value(name = "5")]
    Top,
}

#[derive(Parser)]
#[command(name = "bili", about = "Bilibili API CLI")]
struct Cli {
    #[command(subcommand)]
    action: Option<Action>,

    /// Cookie string (SESSDATA=xx; bili_jct=xx)
    #[arg(long, global = true)]
    cookies: Option<String>,
    /// Output raw JSON (single line)
    #[arg(long, global = true)]
    raw: bool,
    /// Output data field only
    #[arg(long, global = true)]
    compact: bool,
    /// Generate shell completions (bash/zsh/fish/powershell/elvish)
    #[arg(long = "completions", value_name = "SHELL", global = true)]
    completions: Option<String>,
}

#[derive(Subcommand)]
enum Action {
    /// Get video info [requires: --aid|--bvid]
    Info {
        #[arg(long)]
        aid: Option<i64>,
        #[arg(long)]
        bvid: Option<String>,
    },
    /// Popular videos [options: --pn|--ps]
    Popular {
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Related recommendations [requires: --aid|--bvid]
    Related {
        #[arg(long)]
        aid: Option<i64>,
        #[arg(long)]
        bvid: Option<String>,
    },
    /// Homepage recommendations [options: --ps]
    Rcmd {
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Ranking by region [options: --pn|--ps|--rid]
    Hot {
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
        #[arg(long)]
        rid: Option<String>,
    },
    /// User profile [requires: --mid]
    User {
        #[arg(long)]
        mid: i64,
    },
    /// Get current logged-in user's own info
    Mine,
    /// User submitted videos [requires: --mid]
    UserVideos {
        #[arg(long)]
        mid: i64,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// User stats [requires: --mid]
    UserStat {
        #[arg(long)]
        mid: i64,
    },
    /// Following list [requires: --mid]
    Followings {
        #[arg(long)]
        mid: i64,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Follow a user [requires: --fid]
    Follow {
        #[arg(long)]
        fid: i64,
    },
    /// Unfollow a user [requires: --fid]
    Unfollow {
        #[arg(long)]
        fid: i64,
    },
    /// Search [requires: --keyword]
    Search {
        #[arg(long)]
        keyword: String,
        #[arg(long)]
        r#type: Option<SearchType>,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Hot search words [options: --limit]
    HotSearch {
        #[arg(long)]
        limit: Option<i64>,
    },
    /// Comment list [requires: --oid]
    Comments {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        r#type: Option<CommentType>,
        #[arg(long)]
        sort: Option<CommentSort>,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Post a comment [requires: --oid, --message]
    AddComment {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        message: String,
        #[arg(long)]
        r#type: Option<CommentType>,
    },
    /// Favorite folder list [requires: --mid]
    FavList {
        #[arg(long)]
        mid: i64,
    },
    /// Favorite folder contents [requires: --fid]
    FavItems {
        #[arg(long)]
        fid: i64,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Check if video is favorited [requires: --aid]
    HasFavorite {
        #[arg(long)]
        aid: i64,
    },
    /// Danmaku list [requires: --cid]
    Danmaku {
        #[arg(long)]
        cid: i64,
        /// Max segments (default 10)
        #[arg(long)]
        limit: Option<i64>,
    },
    /// Send danmaku [requires: --oid, --message]
    SendDanmaku {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        message: String,
        #[arg(long)]
        mode: Option<DanmakuMode>,
        /// Playback position in ms
        #[arg(long)]
        progress: Option<i64>,
        #[arg(long)]
        aid: Option<i64>,
        #[arg(long)]
        bvid: Option<String>,
    },
    /// Watch history [options: --pn|--ps]
    History {
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Watch later list
    Toview,
    /// Add video to watch later [requires: --aid|--bvid]
    ToviewAdd {
        #[arg(long)]
        aid: Option<i64>,
        #[arg(long)]
        bvid: Option<String>,
    },
    /// Login via QR code
    Login,
    /// Like a video [requires: --aid]
    Like {
        #[arg(long)]
        aid: i64,
    },
    /// Coin a video [requires: --aid]
    Coin {
        #[arg(long)]
        aid: i64,
        /// Coin count (1 or 2)
        #[arg(long)]
        multiply: Option<i64>,
    },
    /// Triple like+coin+fav [requires: --aid]
    Triple {
        #[arg(long)]
        aid: i64,
    },
    /// Subtitle list [requires: --aid, --cid]
    Subtitle {
        #[arg(long)]
        aid: i64,
        #[arg(long)]
        cid: i64,
    },
    /// Subtitle summary [requires: --aid, --cid]
    SubtitleSummary {
        #[arg(long)]
        aid: i64,
        #[arg(long)]
        cid: i64,
        #[arg(long)]
        segment: Option<i64>,
    },
    /// Audio stream URL [requires: --sid]
    Audio {
        #[arg(long)]
        sid: i64,
    },
    /// Video play URL [requires: --aid|--bvid, --cid]
    Playurl {
        #[arg(long)]
        aid: Option<i64>,
        #[arg(long)]
        bvid: Option<String>,
        #[arg(long)]
        cid: Option<i64>,
        #[arg(long)]
        qn: Option<i64>,
        #[arg(long)]
        fnval: Option<i64>,
        #[arg(long)]
        fourk: Option<i64>,
    },
    /// Download video [requires: --bvid]
    Download {
        #[arg(long)]
        bvid: String,
        #[arg(long, default_value = ".")]
        dir: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle --completions early (doesn't need client)
    if let Some(shell) = &cli.completions {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        let shell: clap_complete::Shell = shell.parse().unwrap_or_else(|_| {
            eprintln!("Unknown shell: {shell}. Use bash, zsh, fish, powershell, or elvish.");
            std::process::exit(1);
        });
        generate(shell, &mut cmd, "bili", &mut std::io::stdout());
        return;
    }

    let Some(action) = cli.action else {
        eprintln!("No action specified. Use --help for usage.");
        std::process::exit(1);
    };

    let client = BiliClient::new().expect("Failed to create client");

    // Apply --cookies
    if let Some(cookies) = &cli.cookies {
        let params: Vec<&str> = cookies.split(';').map(|s| s.trim()).collect();
        let url = format!("https://bilibili.com/?{}", params.join("&"));
        if let Err(e) = client.parse_cookies(&url).await {
            eprintln!("Failed to parse cookies: {}", e);
            std::process::exit(1);
        }
    }

    let result = match run(&action, &client).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    print_output(&result, cli.raw, cli.compact);
}

async fn run(
    action: &Action,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match action {
        Action::Info { aid, bvid } => {
            let resp = client.video().info(*aid, bvid.as_deref()).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Popular { pn, ps } => {
            let resp = client.video().popular(*pn, *ps).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Related { aid, bvid } => {
            let resp = client.video().related(*aid, bvid.as_deref()).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Rcmd { ps } => {
            let resp = client.video().rcmd(*ps).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Hot { pn, ps, rid } => {
            let resp = client.video().hot(*pn, *ps, rid.as_deref()).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::User { mid } => {
            let resp = client.user().info(*mid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Mine => {
            let resp = client.user().mine().await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::UserVideos { mid, pn, ps } => {
            let resp = client.user().videos(*mid, *pn, *ps).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::UserStat { mid } => {
            let resp = client.user().stat(*mid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Followings { mid, pn, ps } => {
            let resp = client.user().followings(*mid, *ps, *pn).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Follow { fid } => {
            let resp = client.user().follow(*fid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Unfollow { fid } => {
            let resp = client.user().unfollow(*fid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Search {
            keyword,
            r#type,
            pn,
            ps,
        } => {
            let value = if let Some(search_type) = r#type {
                let api_type = match search_type {
                    SearchType::Video => "video",
                    SearchType::BiliUser => "bili_user",
                    SearchType::MediaBangumi => "media_bangumi",
                    SearchType::MediaFt => "media_ft",
                    SearchType::Live => "live",
                    SearchType::LiveRoom => "live_room",
                    SearchType::LiveUser => "live_user",
                    SearchType::Article => "article",
                    SearchType::Topic => "topic",
                    SearchType::Photo => "photo",
                };
                let items = client.search().query(keyword, *pn, *ps, api_type).await?;
                serde_json::to_value(items)?
            } else {
                let result = client.search().all(keyword, *pn, *ps).await?;
                serde_json::to_value(result)?
            };
            Ok(value)
        }
        Action::HotSearch { limit } => {
            let result = client.search().hot(limit.unwrap_or(50)).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Comments {
            oid,
            r#type,
            sort,
            pn,
            ps,
        } => {
            let comment_type = match r#type.as_ref().unwrap_or(&CommentType::Video) {
                CommentType::Video => 1,
                CommentType::ImageText => 11,
                CommentType::Dynamic => 17,
            };
            let sort_val = sort.as_ref().map(|s| match s {
                CommentSort::Time => 0,
                CommentSort::Like => 2,
                CommentSort::Reply => 3,
            });
            let result = client
                .comment()
                .list(comment_type, *oid, sort_val, *pn, *ps)
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::AddComment {
            oid,
            message,
            r#type,
        } => {
            let comment_type = match r#type.as_ref().unwrap_or(&CommentType::Video) {
                CommentType::Video => 1,
                CommentType::ImageText => 11,
                CommentType::Dynamic => 17,
            };
            let resp = client
                .comment()
                .add(comment_type, *oid, message, None, None, None)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavList { mid } => {
            let result = client.fav().folder_list(*mid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::FavItems { fid, pn, ps } => {
            let result = client
                .fav()
                .resource_list(*fid, None, None, None, *ps, *pn, None)
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::HasFavorite { aid } => {
            let resp = client.fav().has_favorite(*aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Danmaku { cid, limit } => {
            let max_seg = limit.unwrap_or(10);
            let elems = client.danmaku().all_segments(*cid, max_seg).await?;
            Ok(serde_json::to_value(elems)?)
        }
        Action::SendDanmaku {
            oid,
            message,
            mode,
            progress,
            aid,
            bvid,
        } => {
            let mode_val = match mode.as_ref().unwrap_or(&DanmakuMode::Normal) {
                DanmakuMode::Normal => 1,
                DanmakuMode::Bottom => 4,
                DanmakuMode::Top => 5,
            };
            let resp = client
                .danmaku()
                .send(
                    *oid,
                    message,
                    mode_val,
                    progress.unwrap_or(0),
                    *aid,
                    bvid.as_deref(),
                )
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::History { pn, ps } => {
            let result = client.history().list(*pn, *ps).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Toview => {
            let result = client.history().toview_list().await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::ToviewAdd { aid, bvid } => {
            let resp = client.history().toview_add(*aid, bvid.as_deref()).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Login => handle_login(client).await,
        Action::Like { aid } => {
            let resp = client.action().like_video(*aid, true).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Coin { aid, multiply } => {
            let resp = client.action().coin_video(*aid, *multiply, None).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Triple { aid } => {
            let resp = client.action().triple(*aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Subtitle { aid, cid } => {
            let result = client.subtitle().list(*aid, *cid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::SubtitleSummary { aid, cid, segment } => {
            let result = client.subtitle().summary(*aid, *cid, *segment).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Audio { sid } => {
            let result = client.audio().stream_url(*sid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Playurl {
            aid,
            bvid,
            cid,
            qn,
            fnval,
            fourk,
        } => {
            let cid = match cid {
                Some(c) => *c,
                None => {
                    let (_, cid) = client
                        .resolve_bvid(bvid.as_deref().ok_or("requires --bvid")?)
                        .await?;
                    cid
                }
            };
            let resp = client
                .video()
                .playurl(*aid, bvid.as_deref(), cid, *qn, *fnval, *fourk)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Download { bvid, dir } => handle_download(bvid, dir, client).await,
    }
}

async fn handle_download(
    bvid: &str,
    dir: &str,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let info_resp = client.video().info(None, Some(bvid)).await?;
    let info = info_resp.data.ok_or("no video info")?;
    let title = &info.title;
    let pages = &info.pages;

    eprintln!("Downloading: {} ({}, {}P)", title, bvid, pages.len());

    let mut downloaded = Vec::new();

    for page in pages {
        let cid = page.cid;
        let part_name = if pages.len() > 1 {
            format!("{} - P{} {}", title, page.page, page.part)
        } else {
            title.to_string()
        };

        eprintln!("\n  [P{}/{}] {}", page.page, pages.len(), part_name);

        let url_resp = client
            .video()
            .playurl(None, Some(bvid), cid, None, Some(16), None)
            .await?;
        let url_data = url_resp.data.ok_or("no play url")?;

        if let Some(dash) = &url_data.dash {
            let video = dash
                .video
                .iter()
                .max_by_key(|v| v.bandwidth)
                .ok_or("no video stream")?;
            let audio = dash
                .audio
                .iter()
                .max_by_key(|a| a.bandwidth)
                .ok_or("no audio stream")?;

            let video_tmp = format!("{}/{}.video.m4s", dir, part_name);
            let audio_tmp = format!("{}/{}.audio.m4s", dir, part_name);
            let output = format!("{}/{}.mp4", dir, part_name);

            eprintln!("    Video: {}p ({})", video.height, video.codecs);
            eprintln!("    Downloading video stream...");
            download_stream(client, &video.base_url, &video_tmp).await?;

            eprintln!("    Downloading audio stream...");
            download_stream(client, &audio.base_url, &audio_tmp).await?;

            eprintln!("    Merging with ffmpeg...");
            let status = std::process::Command::new("ffmpeg")
                .args([
                    "-i", &video_tmp, "-i", &audio_tmp, "-c", "copy", &output, "-y",
                ])
                .output()?;
            if !status.status.success() {
                let stderr = String::from_utf8_lossy(&status.stderr);
                return Err(format!("ffmpeg failed: {}", stderr).into());
            }

            let _ = std::fs::remove_file(&video_tmp);
            let _ = std::fs::remove_file(&audio_tmp);

            eprintln!("    Saved: {}", output);
            downloaded.push(output);
        } else if let Some(durl) = url_data.durl.first() {
            let output = format!("{}/{}.mp4", dir, part_name);
            download_stream(client, &durl.url, &output).await?;
            eprintln!("    Saved: {}", output);
            downloaded.push(output);
        } else {
            return Err("no playable url found".into());
        }
    }

    Ok(serde_json::json!({"status": "downloaded", "title": title, "files": downloaded}))
}

async fn download_stream(
    client: &BiliClient,
    url: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut resp = client.http().get(url).send().await?;
    let mut file = tokio::fs::File::create(path).await?;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }
    Ok(())
}

async fn handle_login(
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = client.login().qrcode().await?;
    if resp.code != 0 {
        return Err(format!("Failed to generate QR code: {}", resp.message).into());
    }
    let data = resp.data.ok_or("No data in QR code response")?;

    let code = QrCode::with_error_correction_level(data.url.as_bytes(), EcLevel::L)
        .map_err(|e| format!("QR code error: {}", e))?;
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
                println!("Login successful!");
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
