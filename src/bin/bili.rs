use bili::BiliClient;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use qrcode::QrCode;
use qrcode::types::EcLevel;
use reqwest::header::{CONTENT_RANGE, RANGE};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

/// Keep room for `.partN` / `.video.m4s` suffixes and stay below the common
/// 255-**byte** filename limit (Chinese titles use 3 bytes per character).
const MAX_FILENAME_BYTES: usize = 180;

const LOGIN_POLL_INTERVAL: Duration = Duration::from_secs(2);
const LOGIN_TIMEOUT: Duration = Duration::from_secs(180);
const LOGIN_MAX_CONSECUTIVE_ERRORS: u32 = 3;

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
    #[value(name = "video", alias = "1")]
    Video,
    #[value(name = "image-text", alias = "11")]
    ImageText,
    #[value(name = "dynamic", alias = "17")]
    Dynamic,
}

#[derive(Debug, Clone, ValueEnum)]
enum CommentSort {
    #[value(name = "time", alias = "0")]
    Time,
    #[value(name = "like", alias = "2")]
    Like,
    /// 按回复数；懒加载接口不支持该排序，会按热度返回
    #[value(name = "reply", alias = "3")]
    Reply,
}

#[derive(Debug, Clone, ValueEnum)]
enum DanmakuMode {
    #[value(name = "normal", alias = "1")]
    Normal,
    #[value(name = "bottom", alias = "4")]
    Bottom,
    #[value(name = "top", alias = "5")]
    Top,
}

#[derive(Parser)]
#[command(name = "bili", about = "Bilibili API CLI")]
struct Cli {
    #[command(subcommand)]
    action: Option<Action>,

    /// Cookie string (SESSDATA=xx; `bili_jct=xx`)
    #[arg(long, global = true)]
    cookies: Option<String>,
    /// Output raw JSON (single line)
    #[arg(long, global = true, conflicts_with = "compact")]
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
    /// Get video info [requires: --bvid]
    Info {
        #[arg(long)]
        bvid: String,
    },
    /// Popular videos [options: --pn|--ps]
    Popular {
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Related recommendations [requires: --bvid]
    Related {
        #[arg(long)]
        bvid: String,
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
    /// Following list [options: --mid]
    Followings {
        /// Target user mid (defaults to the logged-in user)
        #[arg(long)]
        mid: Option<i64>,
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
    /// Batch follow users [requires: --fids]
    UserBatchFollow {
        /// Comma-separated user mids
        #[arg(long)]
        fids: String,
    },
    /// Batch block users [requires: --fids]
    UserBatchBlock {
        /// Comma-separated user mids
        #[arg(long)]
        fids: String,
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
        /// Pagination offset (`data.cursor.pagination_reply.next_offset` from the previous page)
        #[arg(long)]
        offset: Option<String>,
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
    /// Like or unlike a comment [requires: --oid, --rpid]
    CommentLike {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        rpid: i64,
        #[arg(long)]
        r#type: Option<CommentType>,
        /// 1 = like, 0 = unlike
        #[arg(long, default_value_t = 1)]
        action: i64,
    },
    /// Dislike or undo dislike a comment [requires: --oid, --rpid]
    CommentHate {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        rpid: i64,
        #[arg(long)]
        r#type: Option<CommentType>,
        /// 1 = dislike, 0 = undo
        #[arg(long, default_value_t = 1)]
        action: i64,
    },
    /// Delete a comment [requires: --oid, --rpid]
    CommentDelete {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        rpid: i64,
        #[arg(long)]
        r#type: Option<CommentType>,
    },
    /// Pin or unpin a comment [requires: --oid, --rpid]
    CommentTop {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        rpid: i64,
        #[arg(long)]
        r#type: Option<CommentType>,
        /// 1 = pin, 0 = unpin
        #[arg(long, default_value_t = 1)]
        action: i64,
    },
    /// Report a comment [requires: --oid, --rpid, --reason]
    CommentReport {
        #[arg(long)]
        oid: i64,
        #[arg(long)]
        rpid: i64,
        #[arg(long)]
        reason: i64,
        #[arg(long)]
        content: Option<String>,
        #[arg(long)]
        r#type: Option<CommentType>,
    },
    /// Favorite folder list [options: --mid]
    ///
    /// The returned `id` field is the media id to pass to `fav-items --media-id`;
    /// `fid` is the legacy short id and must not be used there.
    FavList {
        /// Target user mid (defaults to the logged-in user)
        #[arg(long)]
        mid: Option<i64>,
    },
    /// Favorite folder contents [requires: --media-id]
    FavItems {
        /// Favorite folder media id (the `id` field returned by `fav-list`)
        #[arg(long = "media-id", alias = "fid")]
        media_id: i64,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Check if video is favorited [requires: --bvid]
    HasFavorite {
        #[arg(long)]
        bvid: String,
    },
    /// Create a favorite folder [requires: --title]
    FavCreate {
        #[arg(long)]
        title: String,
        #[arg(long)]
        intro: Option<String>,
        /// 0 = public, 1 = private
        #[arg(long)]
        privacy: Option<i64>,
    },
    /// Edit a favorite folder [requires: --media-id, --title]
    FavEdit {
        #[arg(long)]
        media_id: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        intro: Option<String>,
        #[arg(long)]
        privacy: Option<i64>,
    },
    /// Delete favorite folders [requires: --media-ids]
    FavDelete {
        /// Comma-separated favorite folder media ids
        #[arg(long)]
        media_ids: String,
    },
    /// Add a video to a favorite folder [requires: --bvid, --media-id]
    FavAdd {
        #[arg(long)]
        bvid: String,
        #[arg(long)]
        media_id: i64,
    },
    /// Remove a video from a favorite folder [requires: --bvid, --media-id]
    FavRemove {
        #[arg(long)]
        bvid: String,
        #[arg(long)]
        media_id: i64,
    },
    /// Remove all videos from a favorite folder [requires: --media-id]
    FavClean {
        #[arg(long)]
        media_id: i64,
    },
    /// Danmaku list [requires: --bvid]
    Danmaku {
        #[arg(long)]
        bvid: String,
        /// Which part of a multi-part video (default 1)
        #[arg(long)]
        page: Option<i64>,
        /// Max segments (default 10)
        #[arg(long)]
        limit: Option<i64>,
    },
    /// Send danmaku [requires: --bvid, --message]
    SendDanmaku {
        #[arg(long)]
        bvid: String,
        /// Which part of a multi-part video (default 1)
        #[arg(long)]
        page: Option<i64>,
        #[arg(long)]
        message: String,
        #[arg(long)]
        mode: Option<DanmakuMode>,
        /// Playback position in ms
        #[arg(long)]
        progress: Option<i64>,
    },
    /// Recall a danmaku within 2 minutes [requires: --dmid, --cid]
    DanmakuRecall {
        #[arg(long)]
        dmid: i64,
        #[arg(long)]
        cid: i64,
    },
    /// Watch history [options: --max|--view-at|--business|--ps]
    History {
        /// Cursor: last item's id (`data.cursor.max`)
        #[arg(long)]
        max: Option<i64>,
        /// Cursor: last item's timestamp (`data.cursor.view_at`)
        #[arg(long)]
        view_at: Option<i64>,
        /// Cursor: last item's business type (`data.cursor.business`)
        #[arg(long)]
        business: Option<String>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Delete one watch-history record [requires: --aid]
    HistoryDel {
        #[arg(long)]
        aid: i64,
    },
    /// Clear all watch history
    HistoryClear,
    /// Watch later list
    Toview,
    /// Add video to watch later [requires: --bvid]
    ToviewAdd {
        #[arg(long)]
        bvid: String,
    },
    /// Remove a video from watch later [options: --aid|--viewed]
    ToviewDel {
        #[arg(long)]
        aid: Option<i64>,
        /// Remove all already-viewed videos
        #[arg(long)]
        viewed: bool,
    },
    /// Clear watch later list
    ToviewClear,
    /// Login via QR code
    Login,
    /// Like a video [requires: --bvid]
    Like {
        #[arg(long)]
        bvid: String,
    },
    /// Coin a video [requires: --bvid]
    Coin {
        #[arg(long)]
        bvid: String,
        /// Coin count (1 or 2)
        #[arg(long)]
        multiply: Option<i64>,
    },
    /// Triple like+coin+fav [requires: --bvid]
    Triple {
        #[arg(long)]
        bvid: String,
    },
    /// Subtitle list [requires: --bvid]
    Subtitle {
        #[arg(long)]
        bvid: String,
        /// Which part of a multi-part video (default 1)
        #[arg(long)]
        page: Option<i64>,
    },
    /// Subtitle summary [requires: --bvid]
    SubtitleSummary {
        #[arg(long)]
        bvid: String,
        /// Which part of a multi-part video (default 1)
        #[arg(long)]
        page: Option<i64>,
        /// UP主 mid (optional)
        #[arg(long)]
        up_mid: Option<i64>,
    },
    /// Audio stream URL [requires: --sid]
    Audio {
        #[arg(long)]
        sid: i64,
    },
    /// Video play URL [requires: --bvid]
    Playurl {
        #[arg(long)]
        bvid: String,
        /// Which part of a multi-part video (default 1)
        #[arg(long)]
        page: Option<i64>,
        /// Override the auto-resolved cid
        #[arg(long)]
        cid: Option<i64>,
        #[arg(long)]
        qn: Option<i64>,
        #[arg(long)]
        fnval: Option<i64>,
        #[arg(long)]
        fourk: Option<i64>,
    },
    /// Download video [options: --dir|--page|--qn|--fourk|--audio-only]
    Download {
        #[arg(long)]
        bvid: String,
        #[arg(long, default_value = ".")]
        dir: String,
        /// Which part of a multi-part video (default: all parts)
        #[arg(long)]
        page: Option<i64>,
        /// Video quality id (e.g. 80=1080P, 112=1080P+, 120=4K); default: highest available
        #[arg(long)]
        qn: Option<i64>,
        /// Allow 4K quality (passes fourk=1)
        #[arg(long)]
        fourk: bool,
        /// Download only the audio track as .m4a
        #[arg(long)]
        audio_only: bool,
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
        client.parse_cookie_string(cookies);
    }

    // Best-effort device cookie bootstrap: endpoints behind risk control may
    // return 412 without buvid3. Login has its own flow, so skip it there.
    if !matches!(action, Action::Login) {
        let _ = client.bootstrap_buvid().await;
    }

    let result = match run(&action, &client).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            client.flush_cookies();
            std::process::exit(1);
        }
    };

    print_output(&result, cli.raw, cli.compact);

    // API 层面的错误（code != 0）目前也会走到这里；必须用非 0 退出码，
    // 否则脚本无法通过 `bili ... || ...` 判断失败。
    if let Some(code) = result.get("code").and_then(serde_json::Value::as_i64)
        && code != 0
    {
        let message = result
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown error");
        eprintln!("API error {code}: {message}");
        client.flush_cookies();
        std::process::exit(1);
    }
}

fn comment_type_id(value: Option<&CommentType>) -> i64 {
    match value.unwrap_or(&CommentType::Video) {
        CommentType::Video => 1,
        CommentType::ImageText => 11,
        CommentType::Dynamic => 17,
    }
}

fn parse_fids(raw: &str) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    raw.split(',')
        .map(|part| {
            part.trim()
                .parse::<i64>()
                .map_err(|e| format!("invalid fid '{}': {e}", part.trim()).into())
        })
        .collect()
}

#[allow(clippy::too_many_lines)] // a flat command dispatch table
async fn run(
    action: &Action,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match action {
        Action::Info { bvid } => {
            let resp = client.video().info(None, Some(bvid)).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Popular { pn, ps } => {
            let resp = client.video().popular(*pn, *ps).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Related { bvid } => {
            let resp = client.video().related(None, Some(bvid)).await?;
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
            let mid = match mid {
                Some(mid) => *mid,
                None => client.user().get_current_uid()?,
            };
            let resp = client.user().followings(mid, *ps, *pn).await?;
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
        Action::UserBatchFollow { fids } => {
            let fids = parse_fids(fids)?;
            let resp = client.user().batch_follow(&fids).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::UserBatchBlock { fids } => {
            let fids = parse_fids(fids)?;
            let resp = client.user().batch_block(&fids).await?;
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
            offset,
        } => {
            let comment_type = comment_type_id(r#type.as_ref());
            // 懒加载接口的 mode: 2=仅按时间, 3=仅按热度。旧接口的“按回复数”没有对应值。
            let mode = match sort.as_ref().unwrap_or(&CommentSort::Like) {
                CommentSort::Time => 2,
                CommentSort::Like | CommentSort::Reply => 3,
            };
            let pagination = offset
                .as_ref()
                .map(|o| serde_json::json!({ "offset": o }).to_string());
            let result = client
                .comment()
                .list_wbi(comment_type, *oid, Some(mode), pagination.as_deref())
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::AddComment {
            oid,
            message,
            r#type,
        } => {
            let comment_type = comment_type_id(r#type.as_ref());
            let resp = client
                .comment()
                .add(comment_type, *oid, message, None, None, None)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::CommentLike {
            oid,
            rpid,
            r#type,
            action,
        } => {
            let resp = client
                .comment()
                .like(comment_type_id(r#type.as_ref()), *oid, *rpid, Some(*action))
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::CommentHate {
            oid,
            rpid,
            r#type,
            action,
        } => {
            let resp = client
                .comment()
                .hate(comment_type_id(r#type.as_ref()), *oid, *rpid, Some(*action))
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::CommentDelete { oid, rpid, r#type } => {
            let resp = client
                .comment()
                .delete(comment_type_id(r#type.as_ref()), *oid, *rpid)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::CommentTop {
            oid,
            rpid,
            r#type,
            action,
        } => {
            let resp = client
                .comment()
                .top(comment_type_id(r#type.as_ref()), *oid, *rpid, Some(*action))
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::CommentReport {
            oid,
            rpid,
            reason,
            content,
            r#type,
        } => {
            let resp = client
                .comment()
                .report(
                    comment_type_id(r#type.as_ref()),
                    *oid,
                    *rpid,
                    *reason,
                    content.as_deref(),
                )
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavList { mid } => {
            let mid = match mid {
                Some(mid) => *mid,
                None => client.user().get_current_uid()?,
            };
            let result = client.fav().folder_list(mid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::FavItems { media_id, pn, ps } => {
            let result = client
                .fav()
                .resource_list(*media_id, None, None, None, *ps, *pn, None)
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::HasFavorite { bvid } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let resp = client.fav().has_favorite(aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavCreate {
            title,
            intro,
            privacy,
        } => {
            let resp = client
                .fav()
                .folder_add(title, intro.as_deref(), *privacy, None)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavEdit {
            media_id,
            title,
            intro,
            privacy,
        } => {
            let resp = client
                .fav()
                .folder_edit(*media_id, title, intro.as_deref(), *privacy, None)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavDelete { media_ids } => {
            let resp = client.fav().folder_del(media_ids).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavAdd { bvid, media_id } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let resp = client.fav().add(aid, *media_id).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavRemove { bvid, media_id } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let media_id = media_id.to_string();
            let resp = client
                .fav()
                .resource_deal(aid, 2, None, Some(&media_id))
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavClean { media_id } => {
            let resp = client.fav().resource_clean(*media_id).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Danmaku { bvid, page, limit } => {
            let (_, cid) = resolve_ids(client, bvid, *page).await?;
            let max_seg = limit.unwrap_or(10);
            let elems = client.danmaku().all_segments(cid, max_seg).await?;
            Ok(serde_json::to_value(elems)?)
        }
        Action::SendDanmaku {
            bvid,
            page,
            message,
            mode,
            progress,
        } => {
            let (aid, cid) = resolve_ids(client, bvid, *page).await?;
            let mode_val = match mode.as_ref().unwrap_or(&DanmakuMode::Normal) {
                DanmakuMode::Normal => 1,
                DanmakuMode::Bottom => 4,
                DanmakuMode::Top => 5,
            };
            let resp = client
                .danmaku()
                .send(
                    cid,
                    message,
                    mode_val,
                    progress.unwrap_or(0),
                    Some(aid),
                    None,
                )
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::DanmakuRecall { dmid, cid } => {
            let resp = client.danmaku().recall(*dmid, *cid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::History {
            max,
            view_at,
            business,
            ps,
        } => {
            let result = client
                .history()
                .list(*max, *view_at, business.as_deref(), *ps)
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::HistoryDel { aid } => {
            let resp = client.history().delete(*aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::HistoryClear => {
            let resp = client.history().clear().await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Toview => {
            let result = client.history().toview_list().await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::ToviewAdd { bvid } => {
            let resp = client.history().toview_add(None, Some(bvid)).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::ToviewDel { aid, viewed } => {
            let resp = client
                .history()
                .toview_del(*aid, viewed.then_some(true))
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::ToviewClear => {
            let resp = client.history().toview_clear().await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Login => handle_login(client).await,
        Action::Like { bvid } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let resp = client.action().like_video(aid, true).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Coin { bvid, multiply } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let resp = client.action().coin_video(aid, *multiply, None).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Triple { bvid } => {
            let aid = client.resolve_video_ids(bvid).await?.aid;
            let resp = client.action().triple(aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Subtitle { bvid, page } => {
            let (_, cid) = resolve_ids(client, bvid, *page).await?;
            let result = client.subtitle().list(None, Some(bvid), cid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::SubtitleSummary { bvid, page, up_mid } => {
            let (_, cid) = resolve_ids(client, bvid, *page).await?;
            let result = client
                .subtitle()
                .summary(None, Some(bvid), cid, *up_mid)
                .await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Audio { sid } => {
            let result = client.audio().stream_url(*sid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::Playurl {
            bvid,
            page,
            cid,
            qn,
            fnval,
            fourk,
        } => {
            let cid = match cid {
                Some(c) => *c,
                None => resolve_ids(client, bvid, *page).await?.1,
            };
            let resp = client
                .video()
                .playurl(None, Some(bvid), cid, *qn, *fnval, *fourk)
                .await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Download {
            bvid,
            dir,
            page,
            qn,
            fourk,
            audio_only,
        } => {
            let options = DownloadOptions {
                page: *page,
                qn: *qn,
                fourk: *fourk,
                audio_only: *audio_only,
            };
            handle_download(bvid, dir, options, client).await
        }
    }
}

/// Resolve `(aid, cid)` from a `bvid`, selecting a part when `page` is given.
async fn resolve_ids(
    client: &BiliClient,
    bvid: &str,
    page: Option<i64>,
) -> Result<(i64, i64), Box<dyn std::error::Error>> {
    let ids = client.resolve_video_ids(bvid).await?;
    let cid = match page {
        Some(p) => ids
            .cid_of_page(p)
            .ok_or_else(|| format!("no such page: P{p}"))?,
        None => ids.cid,
    };
    Ok((ids.aid, cid))
}

/// Replace characters that are illegal in file names so a video title can be
/// used as a path component safely.
fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = sanitized.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        return "video".to_string();
    }
    // Keep room for the `.partN` / `.video.m4s` suffixes and stay well below
    // the common 255-byte filename limit.
    let truncated: String = trimmed
        .char_indices()
        .take_while(|(index, c)| index + c.len_utf8() <= MAX_FILENAME_BYTES)
        .map(|(_, c)| c)
        .collect();
    let truncated = truncated.trim().trim_end_matches('.').trim();
    if truncated.is_empty() {
        "video".to_string()
    } else {
        truncated.to_string()
    }
}

#[derive(Clone, Copy, Default)]
struct DownloadOptions {
    page: Option<i64>,
    qn: Option<i64>,
    fourk: bool,
    audio_only: bool,
}

#[allow(clippy::too_many_lines)] // sequential download pipeline with explicit error cleanup
async fn handle_download(
    bvid: &str,
    dir: &str,
    options: DownloadOptions,
    client: &BiliClient,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let info_resp = client.video().info(None, Some(bvid)).await?;
    let info = info_resp.data.ok_or("no video info")?;
    let title = &info.title;
    let total_pages = info.pages.len();
    let pages = select_pages(&info.pages, options.page)?;

    eprintln!("Downloading: {title} ({bvid}, {total_pages}P)");

    let dir = std::path::Path::new(dir);
    tokio::fs::create_dir_all(dir).await?;
    let mut downloaded = Vec::new();
    let mut ffmpeg_checked = false;

    for page in pages {
        let cid = page.cid;
        let part_name = if total_pages > 1 {
            sanitize_filename(&format!("{} - P{} {}", title, page.page, page.part))
        } else {
            sanitize_filename(title)
        };

        eprintln!("\n  [P{}/{}] {}", page.page, total_pages, part_name);

        let fnval = if options.fourk { 16 | 128 } else { 16 };
        let url_resp = client
            .video()
            .playurl(
                None,
                Some(bvid),
                cid,
                options.qn,
                Some(fnval),
                options.fourk.then_some(1),
            )
            .await?;
        let url_data = url_resp.data.ok_or("no play url")?;

        if let Some(dash) = &url_data.dash {
            let audio = dash.audio.iter().max_by_key(|a| a.bandwidth);

            if options.audio_only {
                let audio = audio.ok_or("no audio stream")?;
                let audio_tmp = dir.join(format!("{part_name}.audio.m4s"));
                let output = dir.join(format!("{part_name}.m4a"));
                eprintln!("    Audio: {} bps ({})", audio.bandwidth, audio.codecs);
                eprintln!("    Downloading audio stream...");
                if let Err(e) =
                    download_stream(client, &audio.base_url, &audio.backup_url, &audio_tmp).await
                {
                    eprintln!("    Partial file kept for resume: {}", audio_tmp.display());
                    return Err(e);
                }
                tokio::fs::rename(&audio_tmp, &output).await?;
                eprintln!("    Saved: {}", output.display());
                downloaded.push(output.display().to_string());
                continue;
            }

            let video = select_video(&dash.video, options.qn).ok_or("no video stream")?;
            if let Some(qn) = options.qn
                && video.id != qn
            {
                eprintln!(
                    "    Requested qn {qn} is not available; using qn {} ({}p)",
                    video.id, video.height
                );
            }

            let video_tmp = dir.join(format!("{part_name}.video.m4s"));
            let audio_tmp = dir.join(format!("{part_name}.audio.m4s"));
            let output = dir.join(format!("{part_name}.mp4"));

            if !ffmpeg_checked {
                ensure_ffmpeg().await?;
                ffmpeg_checked = true;
            }

            eprintln!("    Video: {}p ({})", video.height, video.codecs);
            eprintln!("    Downloading video + audio streams...");
            let (video_res, audio_res) = tokio::join!(
                download_stream(client, &video.base_url, &video.backup_url, &video_tmp),
                async {
                    match audio {
                        Some(audio) => {
                            download_stream(client, &audio.base_url, &audio.backup_url, &audio_tmp)
                                .await
                        }
                        None => Ok(()),
                    }
                },
            );
            if let Err(e) = video_res {
                eprintln!("    Partial files kept for resume: {}", video_tmp.display());
                return Err(e);
            }
            if let Err(e) = audio_res {
                eprintln!(
                    "    Partial files kept for resume: {} / {}",
                    video_tmp.display(),
                    audio_tmp.display()
                );
                return Err(e);
            }

            eprintln!("    Merging with ffmpeg...");
            let status =
                ffmpeg_merge(&video_tmp, audio.is_some().then_some(&audio_tmp), &output).await;
            let status = match status {
                Ok(status) => status,
                Err(e) => {
                    return Err(e);
                }
            };
            if !status.status.success() {
                let stderr = String::from_utf8_lossy(&status.stderr);
                // A corrupt/HTML partial from an earlier failed download would
                // keep failing on every resume; drop the streams as well.
                cleanup_files(&[&video_tmp, &audio_tmp, &output]).await;
                return Err(format!("ffmpeg failed: {stderr}").into());
            }

            cleanup_files(&[&video_tmp, &audio_tmp]).await;

            eprintln!("    Saved: {}", output.display());
            downloaded.push(output.display().to_string());
        } else if options.audio_only {
            return Err("this video has no DASH audio stream; cannot use --audio-only".into());
        } else if !url_data.durl.is_empty() {
            // The legacy API can split a video into multiple durl segments.
            // Download each one and append them into a single file.
            let output = dir.join(format!("{part_name}.mp4"));
            let mut out = tokio::fs::File::create(&output).await?;
            for (index, segment) in url_data.durl.iter().enumerate() {
                eprintln!(
                    "    Downloading segment {}/{}...",
                    index + 1,
                    url_data.durl.len()
                );
                let part = dir.join(format!("{part_name}.part{index}"));
                if let Err(e) =
                    download_stream(client, &segment.url, &segment.backup_url, &part).await
                {
                    drop(out);
                    cleanup_files(&[&part, &output]).await;
                    return Err(e);
                }
                let mut part_file = tokio::fs::File::open(&part).await?;
                tokio::io::copy(&mut part_file, &mut out).await?;
                drop(part_file);
                let _ = tokio::fs::remove_file(&part).await;
            }
            out.flush().await?;
            eprintln!("    Saved: {}", output.display());
            downloaded.push(output.display().to_string());
        } else {
            return Err("no playable url found".into());
        }
    }

    Ok(serde_json::json!({"status": "downloaded", "title": title, "files": downloaded}))
}

/// Pick the requested quality, or the highest-bandwidth stream.
fn select_video(
    videos: &[bili::types::DashVideoItem],
    qn: Option<i64>,
) -> Option<&bili::types::DashVideoItem> {
    if let Some(qn) = qn
        && let Some(video) = videos
            .iter()
            .filter(|video| video.id == qn)
            .max_by_key(|video| video.bandwidth)
    {
        return Some(video);
    }
    videos.iter().max_by_key(|video| video.bandwidth)
}

/// Fail fast if ffmpeg is missing, before downloading hundreds of megabytes.
async fn ensure_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    match tokio::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(format!(
            "ffmpeg is not usable: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("ffmpeg not found; install ffmpeg to download DASH video".into())
        }
        Err(e) => Err(e.into()),
    }
}

/// Select one part (or all parts) of a video.
fn select_pages(
    pages: &[bili::types::VideoPage],
    page: Option<i64>,
) -> Result<Vec<&bili::types::VideoPage>, String> {
    match page {
        Some(page) => {
            let selected: Vec<_> = pages.iter().filter(|part| part.page == page).collect();
            if selected.is_empty() {
                Err(format!("no such page: P{page}"))
            } else {
                Ok(selected)
            }
        }
        None => Ok(pages.iter().collect()),
    }
}

/// Run ffmpeg to remux the downloaded streams into `output`.
async fn ffmpeg_merge(
    video: &std::path::Path,
    audio: Option<&std::path::PathBuf>,
    output: &std::path::Path,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut cmd = tokio::process::Command::new("ffmpeg");
    cmd.arg("-y").arg("-nostdin").arg("-i").arg(video);
    if let Some(audio) = audio {
        cmd.arg("-i").arg(audio);
    }
    cmd.arg("-map").arg("0:v:0");
    if audio.is_some() {
        cmd.arg("-map").arg("1:a:0");
    }
    cmd.arg("-c").arg("copy").arg(output);

    match cmd.output().await {
        Ok(status) => Ok(status),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("ffmpeg not found; install ffmpeg to merge DASH video/audio streams".into())
        }
        Err(e) => Err(e.into()),
    }
}

async fn cleanup_files(paths: &[&std::path::Path]) {
    for path in paths {
        let _ = tokio::fs::remove_file(path).await;
    }
}

/// Parse `Content-Range: bytes <start>-<end>/<total>`.
fn parse_content_range(value: &str) -> Option<(u64, u64)> {
    let rest = value.strip_prefix("bytes ")?;
    let (range, total) = rest.split_once('/')?;
    let (start, _end) = range.split_once('-')?;
    Some((start.parse().ok()?, total.parse().ok()?))
}

async fn download_stream(
    client: &BiliClient,
    url: &str,
    backup_urls: &[String],
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match download_one(client, url, path).await {
        Ok(()) => Ok(()),
        Err(first_error) => {
            for backup in backup_urls {
                if download_one(client, backup, path).await.is_ok() {
                    return Ok(());
                }
            }
            Err(first_error)
        }
    }
}

async fn download_one(
    client: &BiliClient,
    url: &str,
    path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resume a previous partial download when possible. The CDN URLs are
    // signed and can expire, so a failed download is resumed by re-fetching a
    // fresh play URL in the next run.
    let existing = tokio::fs::metadata(path)
        .await
        .map_or(0, |metadata| metadata.len());
    let mut request = client.download_http().get(url);
    if existing > 0 {
        request = request.header(RANGE, format!("bytes={existing}-"));
    }

    // Use the download client (no total timeout) and fail on 403/404 instead
    // of writing an HTML error page into the .m4s file.
    let resp = request.send().await?;
    let status = resp.status();
    if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE && existing > 0 {
        return Err(format!(
            "partial file is already complete or corrupt: {}; delete it and retry",
            path.display()
        )
        .into());
    }
    let resp = resp.error_for_status()?;
    let content_range = resp
        .headers()
        .get(CONTENT_RANGE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_content_range);
    let resuming = status == reqwest::StatusCode::PARTIAL_CONTENT && existing > 0;
    if resuming && content_range.is_none_or(|(start, _)| start != existing) {
        return Err(format!(
            "server returned an unexpected range for {}; delete it and retry",
            path.display()
        )
        .into());
    }
    let expected_total = if resuming {
        content_range.map(|(_, total)| total)
    } else {
        resp.content_length()
    };

    let mut resp = resp;
    let mut file = if resuming {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .await?
    } else {
        tokio::fs::File::create(path).await?
    };
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    if let Some(expected) = expected_total {
        let actual = tokio::fs::metadata(path).await?.len();
        if actual != expected {
            return Err(format!(
                "incomplete download: {actual}/{expected} bytes (rerun to resume)"
            )
            .into());
        }
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
        .map_err(|e| format!("QR code error: {e}"))?;
    println!("Scan the QR code below to login:");
    println!(
        "{}",
        code.render::<qrcode::render::unicode::Dense1x2>()
            .quiet_zone(false)
            .build()
    );

    let deadline = tokio::time::Instant::now() + LOGIN_TIMEOUT;
    let mut consecutive_errors = 0u32;
    let mut announced_scan = false;

    loop {
        if tokio::time::Instant::now() >= deadline {
            return Err("Login timed out; run `bili login` again".into());
        }
        tokio::time::sleep(LOGIN_POLL_INTERVAL).await;

        let status = match client.login().qrcode_status(&data.qrcode_key).await {
            Ok(status) => {
                consecutive_errors = 0;
                status
            }
            Err(error) => {
                consecutive_errors += 1;
                if consecutive_errors >= LOGIN_MAX_CONSECUTIVE_ERRORS {
                    return Err(format!(
                        "Login polling failed {consecutive_errors} times in a row: {error}"
                    )
                    .into());
                }
                eprintln!(
                    "Login poll failed ({consecutive_errors}/{LOGIN_MAX_CONSECUTIVE_ERRORS}): {error}; retrying..."
                );
                continue;
            }
        };

        if status.code != 0 {
            return Err(format!("Login failed: {}", status.message).into());
        }
        let s = status.data.ok_or("No data in status response")?;

        match s.code {
            0 => {
                let Some(url) = s.url else {
                    return Err(
                        "Login succeeded but the response contains no cookie URL; please retry"
                            .into(),
                    );
                };
                client.parse_cookies(&url)?;
                if !client.is_logged_in() {
                    return Err(
                        "Login returned cookies but SESSDATA is missing; please retry".into(),
                    );
                }
                client.flush_cookies();
                println!("Login successful!");
                return Ok(serde_json::json!({"status": "success"}));
            }
            86090 => {
                if !announced_scan {
                    eprintln!("QR code scanned, waiting for confirmation...");
                    announced_scan = true;
                }
            }
            86101 => {}
            86038 => {
                return Err("QR code expired, please try again".into());
            }
            other => {
                return Err(format!("Unexpected login status: {other}").into());
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

#[cfg(test)]
mod tests {
    use super::{
        MAX_FILENAME_BYTES, download_one, parse_content_range, parse_fids, sanitize_filename,
        select_pages, select_video,
    };
    use bili::BiliClient;
    use bili::types::{DashVideoItem, VideoPage};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn pages() -> Vec<VideoPage> {
        vec![
            VideoPage {
                page: 1,
                ..VideoPage::default()
            },
            VideoPage {
                page: 2,
                ..VideoPage::default()
            },
        ]
    }

    fn temp_cookie_path() -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bili_bin_test_{ts}.json"))
    }

    fn temp_stream_path() -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("bili_stream_test_{ts}.m4s"))
    }

    #[test]
    fn sanitize_filename_replaces_illegal_characters() {
        let name = sanitize_filename(r#"a/b:c*?"<>|"#);
        assert_eq!(name, "a_b_c______");
        assert!(
            !name
                .chars()
                .any(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        );
    }

    #[test]
    fn sanitize_filename_trims_and_falls_back() {
        assert_eq!(sanitize_filename("  ..  "), "video");
        assert_eq!(sanitize_filename("name... "), "name");
    }

    #[test]
    fn sanitize_filename_truncates_long_titles() {
        let name = sanitize_filename(&"字".repeat(300));
        assert!(name.len() <= MAX_FILENAME_BYTES);
        assert_eq!(name.chars().count(), MAX_FILENAME_BYTES / 3);
    }

    #[test]
    fn select_pages_supports_one_or_all_parts() {
        let parts = pages();
        assert_eq!(select_pages(&parts, None).unwrap().len(), 2);
        assert_eq!(select_pages(&parts, Some(2)).unwrap()[0].page, 2);
        assert!(select_pages(&parts, Some(99)).is_err());
    }

    #[test]
    fn parses_fids() {
        assert_eq!(parse_fids("1, 2,3").unwrap(), vec![1, 2, 3]);
        assert!(parse_fids("1,x").is_err());
    }

    #[test]
    fn parses_content_range() {
        assert_eq!(parse_content_range("bytes 3-5/6"), Some((3, 6)));
        assert_eq!(parse_content_range("bytes */6"), None);
        assert_eq!(parse_content_range("garbage"), None);
    }

    #[test]
    fn select_video_honors_requested_quality() {
        let videos = vec![
            DashVideoItem {
                id: 80,
                bandwidth: 1_000,
                ..DashVideoItem::default()
            },
            DashVideoItem {
                id: 120,
                bandwidth: 3_000,
                ..DashVideoItem::default()
            },
        ];
        assert_eq!(select_video(&videos, None).unwrap().id, 120);
        assert_eq!(select_video(&videos, Some(80)).unwrap().id, 80);
        // Unavailable quality falls back to the highest-bandwidth stream.
        assert_eq!(select_video(&videos, Some(999)).unwrap().id, 120);
    }

    #[tokio::test]
    async fn download_resumes_a_partial_file() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/stream"))
            .respond_with(
                ResponseTemplate::new(206)
                    .insert_header("content-range", "bytes 3-5/6")
                    .set_body_string("def"),
            )
            .mount(&server)
            .await;

        let cookie_path = temp_cookie_path();
        let stream_path = temp_stream_path();
        std::fs::write(&stream_path, b"abc").unwrap();
        let client = BiliClient::builder()
            .cookie_path(cookie_path.clone())
            .build()
            .unwrap();

        download_one(&client, &format!("{}/stream", server.uri()), &stream_path)
            .await
            .expect("resume");
        assert_eq!(std::fs::read(&stream_path).unwrap(), b"abcdef");

        drop(client);
        let _ = std::fs::remove_file(cookie_path);
        let _ = std::fs::remove_file(stream_path);
    }

    #[tokio::test]
    async fn download_restarts_when_server_ignores_range() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/stream"))
            .respond_with(ResponseTemplate::new(200).set_body_string("xyz"))
            .mount(&server)
            .await;

        let cookie_path = temp_cookie_path();
        let stream_path = temp_stream_path();
        std::fs::write(&stream_path, b"abc").unwrap();
        let client = BiliClient::builder()
            .cookie_path(cookie_path.clone())
            .build()
            .unwrap();

        download_one(&client, &format!("{}/stream", server.uri()), &stream_path)
            .await
            .expect("restart");
        assert_eq!(std::fs::read(&stream_path).unwrap(), b"xyz");

        drop(client);
        let _ = std::fs::remove_file(cookie_path);
        let _ = std::fs::remove_file(stream_path);
    }
}
