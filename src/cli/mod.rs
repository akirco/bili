//! Command-line interface: argument definitions and dispatch.
//!
//! Compiled only with the `cli` feature. [`main_flow`] contains everything
//! the `cli/bin/bili.rs` entry point does beyond argument parsing, so the
//! binary stays thin.
//!
//! The whole CLI lives here — argument parsing, dispatch, result rendering,
//! the QR login flow and the download engine it drives ([`download`]).

use self::args::parse_fids;
use self::download::DownloadOptions;
use self::output::OutputMode;
use crate::BiliClient;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::generate;
use qrcode::QrCode;
use qrcode::types::EcLevel;

pub mod args;
pub mod download;
pub mod login_flow;
pub mod output;

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
pub struct Cli {
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
    /// Search live rooms [requires: --keyword]
    LiveSearch {
        #[arg(long)]
        keyword: String,
        #[arg(long)]
        pn: Option<i64>,
        #[arg(long)]
        ps: Option<i64>,
    },
    /// Live stream URLs [requires: --room]
    LiveMedia {
        /// Live room id (the long or short id both work)
        #[arg(long)]
        room: i64,
        /// Quality id, 0 = server default
        #[arg(long)]
        qn: Option<i64>,
        /// Only print the playable URLs, one per line
        #[arg(long)]
        urls: bool,
        /// Prefer the HTTP-FLV stream when printing --urls
        #[arg(long)]
        flv: bool,
    },
    /// Live category (area) list
    LiveCategories {
        /// Only print child areas, skipping the parent rows
        #[arg(long)]
        flat: bool,
    },
    /// Live rooms in one category [requires: --parent, --area]
    ///
    /// Needs login: the upstream endpoint returns -352 for anonymous requests.
    LiveArea {
        /// Parent area id (from `live-categories`)
        #[arg(long)]
        parent: i64,
        /// Child area id; 0 = the whole parent area
        #[arg(long, default_value_t = 0)]
        area: i64,
        #[arg(long)]
        pn: Option<i64>,
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
fn comment_type_id(value: Option<&CommentType>) -> i64 {
    match value.unwrap_or(&CommentType::Video) {
        CommentType::Video => 1,
        CommentType::ImageText => 11,
        CommentType::Dynamic => 17,
    }
}
// A flat command dispatch table: every arm is a one-to-three-line forward to a
// client method, so splitting it would only move the lines around. Grouping it
// per domain is tracked as a follow-up (`cli/video.rs`, `cli/user.rs`, ...).
#[allow(clippy::too_many_lines)]
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
            let aid = client.video().resolve_ids(bvid).await?.aid;
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
            let aid = client.video().resolve_ids(bvid).await?.aid;
            let resp = client.fav().add(aid, *media_id).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::FavRemove { bvid, media_id } => {
            let aid = client.video().resolve_ids(bvid).await?.aid;
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
            let (_, cid) = download::resolve_for_page(client, bvid, *page).await?;
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
            let (aid, cid) = download::resolve_for_page(client, bvid, *page).await?;
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
        Action::Login => {
            login_flow::qr_login(client, render_qr, |msg| eprintln!("{msg}")).await?;
            Ok(serde_json::json!({"status": "success"}))
        }
        Action::Like { bvid } => {
            let aid = client.video().resolve_ids(bvid).await?.aid;
            let resp = client.action().like_video(aid, true).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Coin { bvid, multiply } => {
            let aid = client.video().resolve_ids(bvid).await?.aid;
            let resp = client.action().coin_video(aid, *multiply, None).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Triple { bvid } => {
            let aid = client.video().resolve_ids(bvid).await?.aid;
            let resp = client.action().triple(aid).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::Subtitle { bvid, page } => {
            let (_, cid) = download::resolve_for_page(client, bvid, *page).await?;
            let result = client.subtitle().list(None, Some(bvid), cid).await?;
            Ok(serde_json::to_value(result)?)
        }
        Action::SubtitleSummary { bvid, page, up_mid } => {
            let (_, cid) = download::resolve_for_page(client, bvid, *page).await?;
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
        Action::LiveSearch { keyword, pn, ps } => {
            let resp = client.live().search(keyword, *pn, *ps).await?;
            Ok(serde_json::to_value(resp)?)
        }
        Action::LiveMedia {
            room,
            qn,
            urls,
            flv,
        } => {
            let resp = client.live().media(*room, *qn).await?;
            if !urls {
                return Ok(serde_json::to_value(resp)?);
            }
            // `--urls` 是给人/脚本直接拿去 ffmpeg 的，所以只吐地址本身，
            // 不再包一层 JSON；没有流时返回空数组，让 check_api_error 仍能看 code。
            let Some(info) = resp.data.as_ref() else {
                return Ok(serde_json::to_value(resp)?);
            };
            let mut streams = info.streams();
            if *flv {
                streams.sort_by_key(|s| !s.is_flv());
            }
            let urls: Vec<&str> = streams.iter().map(|s| s.url.as_str()).collect();
            Ok(serde_json::json!({
                "code": resp.code,
                "message": resp.message,
                "live_status": info.live_status,
                "urls": urls,
            }))
        }
        Action::LiveCategories { flat } => {
            let resp = client.live().categories().await?;
            if !flat {
                return Ok(serde_json::to_value(resp)?);
            }
            let Some(list) = resp.data.as_ref() else {
                return Ok(serde_json::to_value(resp)?);
            };
            let rows: Vec<serde_json::Value> = list
                .flat()
                .into_iter()
                .map(|(parent, child)| match child {
                    Some(child) => serde_json::json!({
                        "parent_id": parent.id,
                        "parent_name": parent.name,
                        "id": child.id,
                        "name": child.name,
                        "area_type": child.area_type,
                    }),
                    None => serde_json::json!({
                        "parent_id": parent.id,
                        "parent_name": parent.name,
                        "id": serde_json::Value::Null,
                        "name": parent.name,
                        "area_type": serde_json::Value::Null,
                    }),
                })
                .collect();
            Ok(serde_json::json!({
                "code": resp.code,
                "message": resp.message,
                "areas": rows,
            }))
        }
        Action::LiveArea { parent, area, pn } => {
            let resp = client.live().areas(*parent, *area, *pn).await?;
            Ok(serde_json::to_value(resp)?)
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
                None => download::resolve_for_page(client, bvid, *page).await?.1,
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
            Ok(download::handle_download(bvid, dir, options, client).await?)
        }
    }
}

/// Print the QR code as dense unicode blocks so it can be scanned from a
/// terminal.
fn render_qr(url: &str) -> Result<(), login_flow::LoginError> {
    let code = QrCode::with_error_correction_level(url.as_bytes(), EcLevel::L)
        .map_err(|e| login_flow::LoginError::Render(e.to_string()))?;
    println!("Scan the QR code below to login:");
    println!(
        "{}",
        code.render::<qrcode::render::unicode::Dense1x2>()
            .quiet_zone(false)
            .build()
    );
    Ok(())
}

/// Run the CLI and return the process exit code.
///
/// This is the whole of what `main` used to do: argument post-processing,
/// device-cookie bootstrap, dispatch, output rendering and the exit-code
/// policy for API-level errors.
pub async fn main_flow(cli: Cli) -> i32 {
    if let Some(shell) = &cli.completions {
        let mut cmd = <Cli as clap::CommandFactory>::command();
        let Ok(shell) = shell.parse::<clap_complete::Shell>() else {
            eprintln!("Unknown shell: {shell}. Use bash, zsh, fish, powershell, or elvish.");
            return 1;
        };
        generate(shell, &mut cmd, "bili", &mut std::io::stdout());
        return 0;
    }

    let Some(action) = cli.action else {
        eprintln!("No action specified. Use --help for usage.");
        return 1;
    };

    let client = match BiliClient::new() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };

    if let Some(cookies) = &cli.cookies {
        client.parse_cookie_string(cookies);
    }

    // Best-effort device cookie bootstrap: endpoints behind risk control may
    // return 412 without buvid3. Login has its own flow, so skip it there.
    if !matches!(action, Action::Login) {
        let _ = client.bootstrap_buvid().await;
    }

    let mode = if cli.raw {
        OutputMode::Raw
    } else if cli.compact {
        OutputMode::Compact
    } else {
        OutputMode::Pretty
    };

    let result = match run(&action, &client).await {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error: {e}");
            client.flush_cookies();
            return 1;
        }
    };

    output::print_value(&result, mode);

    // API 层面的错误（code != 0）也会走到这里；必须用非 0 退出码，
    // 否则脚本无法通过 `bili ... || ...` 判断失败。
    if let Err(e) = output::check_api_error(&result) {
        eprintln!("{e}");
        client.flush_cookies();
        return 1;
    }

    0
}
