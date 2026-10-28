//! 契约测试：用 B 站接口文档中的真实响应片段验证各类型能正确反序列化。
//!
//! 这些测试的字段名全部来自 `docs/` 中的响应示例，目的是防止
//! `#[serde(rename_all = ...)]` / 字段名拼写错误导致字段被静默降级为默认值。

#![allow(
    clippy::items_after_statements,
    clippy::needless_raw_string_hashes,
    clippy::too_many_lines
)]

use bili::types::{
    AudioStreamUrl, CommentListWbi, FavFolderInfo, FavResourceListResult, HistoryResult,
    SearchResultItem, SearchTypeResult, SearchVideo, UserInfo, UserStat, UserVideoListResult,
    VideoRecommendItem,
};

#[test]
fn user_info_uses_snake_case_fields() {
    let json = r##"{
        "mid": 2,
        "name": "碧诗",
        "sex": "男",
        "face": "https://i2.hdslb.com/bfs/face/x.jpg",
        "face_nft": 0,
        "face_nft_type": 0,
        "sign": "sign",
        "rank": 20000,
        "level": 6,
        "jointime": 0,
        "moral": 0,
        "silence": 0,
        "coins": 0,
        "fans_badge": true,
        "fans_medal": {
            "show": true,
            "wear": true,
            "medal": {
                "uid": 2,
                "target_id": 548076,
                "medal_id": 32525,
                "level": 28,
                "guard_level": 3,
                "medal_name": "桜樱怪",
                "medal_color": 398668,
                "intimacy": 25364,
                "next_intimacy": 160000,
                "day_limit": 250000,
                "today_feed": 2382,
                "medal_color_start": 398668,
                "medal_color_end": 6850801,
                "medal_color_border": 6809855,
                "is_lighted": 1,
                "light_status": 1,
                "wearing_status": 1,
                "score": 50185364
            }
        },
        "official": { "role": 2, "title": "站长", "desc": "", "type": 0 },
        "vip": {
            "type": 2,
            "status": 1,
            "due_date": 3979555200000,
            "vip_pay_type": 0,
            "theme_type": 0,
            "label": {
                "path": "",
                "text": "十年大会员",
                "label_theme": "ten_annual_vip",
                "text_color": "#FFFFFF",
                "bg_style": 1,
                "bg_color": "#FB7299",
                "border_color": "",
                "use_img_label": true,
                "img_label_uri_hans": "",
                "img_label_uri_hant": "",
                "img_label_uri_hans_static": "",
                "img_label_uri_hant_static": ""
            },
            "avatar_subscript": 1,
            "nickname_color": "#FB7299",
            "role": 7,
            "avatar_subscript_url": "",
            "tv_vip_status": 1,
            "tv_vip_pay_type": 1,
            "tv_due_date": 2003500800,
            "avatar_icon": { "icon_type": 1, "icon_resource": {} }
        },
        "pendant": {
            "pid": 32257,
            "name": "EveOneCat2",
            "image": "img",
            "expire": 0,
            "image_enhance": "enhance",
            "image_enhance_frame": "frame",
            "n_pid": 32257
        },
        "nameplate": {
            "nid": 10,
            "name": "见习偶像",
            "image": "img",
            "image_small": "small",
            "level": "普通勋章",
            "condition": "条件"
        },
        "user_honour_info": { "mid": 0, "colour": null, "tags": [], "is_latest_100honour": 0 },
        "is_followed": true,
        "top_photo": "http://i0.hdslb.com/bfs/space/top.png",
        "theme": {},
        "sys_notice": {},
        "live_room": {
            "roomStatus": 1,
            "liveStatus": 0,
            "url": "https://live.bilibili.com/1024",
            "title": "试图恰鸡",
            "cover": "cover",
            "roomid": 1024,
            "roundStatus": 0,
            "broadcast_type": 0,
            "watched_show": {
                "switch": true,
                "num": 3,
                "text_small": "3",
                "text_large": "3人看过",
                "icon": "icon",
                "icon_location": "",
                "icon_web": "icon_web"
            }
        },
        "birthday": "09-19",
        "school": { "name": "" },
        "profession": { "name": "", "department": "", "title": "", "is_show": 0 },
        "tags": null,
        "series": { "user_upgrade_status": 3, "show_upgrade_window": false },
        "is_senior_member": 0,
        "mcn_info": null,
        "gaia_res_type": 0,
        "gaia_data": null,
        "is_risk": false,
        "elec": { "show_info": { "show": true, "state": 1, "title": "", "icon": "", "jump_url": "?oid=2" } },
        "contract": { "is_display": true, "is_follow_display": false },
        "certificate_show": true,
        "name_render": null
    }"##;

    let info: UserInfo = serde_json::from_str(json).expect("UserInfo should deserialize");
    // Serialize 也必须输出 snake_case，否则 CLI 的 --raw/--compact 输出会和其它命令不一致。
    let out = serde_json::to_value(&info).unwrap();

    assert_eq!(info.face_nft, 0);
    assert_eq!(info.top_photo, "http://i0.hdslb.com/bfs/space/top.png");
    assert!(info.is_followed);
    assert!(info.certificate_show);
    assert!(info.contract.expect("contract").is_display);

    let medal = info.fans_medal.expect("fans_medal").medal;
    assert_eq!(medal.guard_level, Some(3));
    assert_eq!(medal.next_intimacy, 160_000);
    assert_eq!(medal.medal_color_end, 6_850_801);

    let vip = info.vip.expect("vip");
    assert_eq!(vip.due_date, 3_979_555_200_000);
    assert_eq!(vip.label.label_theme, "ten_annual_vip");
    assert_eq!(vip.tv_due_date, 2_003_500_800);

    let live = info.live_room.expect("live_room");
    assert_eq!(live.room_status, 1);
    assert_eq!(live.live_status, 0);
    assert_eq!(live.round_status, 0);
    assert_eq!(live.watched_show.text_large, "3人看过");

    assert_eq!(
        info.user_honour_info.expect("honour").is_latest_100honour,
        Some(0)
    );

    // Serialize 也必须输出 snake_case，否则 CLI 的 --raw/--compact 输出会和其它命令不一致。
    assert_eq!(out["is_followed"], true);
    assert!(out.get("isFollowed").is_none());
}

#[test]
fn fav_folder_info_and_resource_list_use_snake_case() {
    let json = r##"{
        "info": {
            "id": 1,
            "fid": 2,
            "mid": 3,
            "attr": 0,
            "title": "收藏夹",
            "cover": "cover",
            "upper": { "mid": 4, "name": "籽岷", "face": "face", "followed": true, "vip_type": 2, "vip_statue": 1 },
            "cover_type": 2,
            "cnt_info": { "collect": 3393, "play": 184822, "thumb_up": 3918, "share": 44 },
            "type": 11,
            "intro": "intro",
            "ctime": 1598884758,
            "mtime": 1598884758,
            "state": 0,
            "fav_state": 0,
            "like_state": 0,
            "media_count": 28
        },
        "medias": [{
            "id": 371494037,
            "type": 2,
            "title": "猛 男 生 存",
            "cover": "cover",
            "intro": "intro",
            "page": 1,
            "duration": 546,
            "upper": { "mid": 686127, "name": "籽岷", "face": "face" },
            "attr": 0,
            "cnt_info": { "collect": 11256, "play": 1638040, "danmaku": 7697 },
            "link": "bilibili://video/371494037",
            "ctime": 1595690513,
            "pubtime": 1595690513,
            "fav_time": 1598884777,
            "bv_id": "BV1CZ4y1T7gC",
            "bvid": "BV1CZ4y1T7gC",
            "season": null
        }],
        "has_more": true,
        "ttl": 1
    }"##;

    let data: FavResourceListResult =
        serde_json::from_str(json).expect("fav list should deserialize");
    assert_eq!(data.info.media_count, 28);
    assert_eq!(data.info.cnt_info.expect("cnt_info").thumb_up, 3918);
    assert_eq!(data.info.upper.expect("upper").vip_type, Some(2));
    assert_eq!(data.medias[0].cnt_info.play, 1_638_040);
    assert_eq!(data.medias[0].fav_time, 1_598_884_777);
    assert!(data.medias[0].season.is_none());
    assert!(data.has_more);
}

#[test]
fn history_result_uses_cursor_and_snake_case() {
    let json = r##"{
        "list": [{
            "title": "标题",
            "long_title": "副标题",
            "cover": "cover",
            "covers": ["a"],
            "uri": "bilibili://video/1",
            "history": {
                "oid": 1,
                "epid": 0,
                "bvid": "BV1",
                "page": 1,
                "cid": 2,
                "part": "P1",
                "business": "archive",
                "dt": 3
            },
            "videos": 1,
            "author_name": "UP主",
            "author_face": "face",
            "author_mid": 9,
            "view_at": 1592985807,
            "progress": -1,
            "badge": "badge",
            "show_title": "分P标题",
            "duration": 100,
            "current": "",
            "total": 0,
            "new_desc": "",
            "is_finish": 0,
            "is_fav": 0,
            "kid": 0,
            "tag_name": "子分区",
            "live_status": 0
        }],
        "has_more": true,
        "cursor": { "max": 1, "view_at": 2, "business": "archive", "ps": 20 },
        "tab": [{ "type": "all", "name": "全部" }],
        "business": "",
        "ps": 20
    }"##;

    let result: HistoryResult = serde_json::from_str(json).expect("HistoryResult");
    assert_eq!(result.list[0].long_title, "副标题");
    assert_eq!(result.list[0].author_name, "UP主");
    assert_eq!(result.list[0].view_at, 1_592_985_807);
    assert_eq!(result.list[0].show_title, "分P标题");
    assert_eq!(result.list[0].tag_name, "子分区");
    assert!(result.has_more);
    assert_eq!(result.cursor.view_at, 2);
    assert_eq!(result.tab[0].tab_type, "all");
}

#[test]
fn comment_list_wbi_keeps_snake_case_and_vip_camel_case() {
    let json = r##"{
        "cursor": {
            "all_count": 10,
            "is_begin": true,
            "prev": 0,
            "next": 0,
            "is_end": false,
            "mode": 3,
            "support_mode": null,
            "name": "",
            "pagination_reply": { "next_offset": "" },
            "session_id": ""
        },
        "replies": [{
            "rpid": 1,
            "oid": 2,
            "type": 1,
            "mid": 3,
            "root": 0,
            "parent": 0,
            "dialog": 0,
            "count": 0,
            "rcount": 0,
            "state": 0,
            "fansgrade": 0,
            "attr": 0,
            "ctime": 1,
            "rpid_str": "1",
            "root_str": "0",
            "parent_str": "0",
            "like": 1,
            "action": 0,
            "member": {
                "mid": "3",
                "uname": "用户",
                "sex": "保密",
                "sign": "",
                "avatar": "avatar",
                "rank": "10000",
                "DisplayRank": "0",
                "face_nft_new": 0,
                "is_senior_member": 0,
                "level_info": { "current_level": 4, "current_min": 0, "current_exp": 0, "next_exp": 0 },
                "pendant": {},
                "nameplate": {},
                "official_verify": { "type": -1, "desc": "" },
                "vip": {
                    "vipType": 1,
                    "vipDueDate": 1648051200000,
                    "dueRemark": "",
                    "accessStatus": 0,
                    "vipStatus": 1,
                    "vipStatusWarn": "",
                    "themeType": 0,
                    "label": { "path": "", "text": "大会员", "label_theme": "vip", "text_color": "#FFFFFF", "bg_style": 0, "bg_color": "", "border_color": "" },
                    "avatar_subscript": 1,
                    "avatar_subscript_url": "url",
                    "nickname_color": ""
                },
                "fans_detail": null,
                "following": 0,
                "is_followed": 0,
                "user_sailing": { "pendant": null, "cardbg": null, "cardbg_with_focus": null },
                "is_contractor": false,
                "contract_desc": ""
            },
            "content": {
                "message": "评论内容",
                "plat": 0,
                "device": "",
                "members": [],
                "emote": {},
                "jump_url": { "rick叔": { "title": "nested object" } },
                "max_line": 6
            },
            "replies": [],
            "assist": 0,
            "folder": { "has_folded": false, "is_folded": false, "rule": "" },
            "up_action": { "like": false, "reply": false },
            "show_follow": false,
            "invisible": false,
            "reply_control": { "time_desc": "6小时前发布" }
        }],
        "top": {},
        "folder": {},
        "config": {
            "showadmin": 1,
            "showentry": 1,
            "showfloor": 0,
            "showtopic": 1,
            "show_up_flag": true,
            "read_only": false,
            "show_del_log": false
        },
        "upper": {},
        "show_bvid": false,
        "control": {}
    }"##;

    let list: CommentListWbi = serde_json::from_str(json).expect("CommentListWbi");
    assert_eq!(list.cursor.all_count, 10);

    let reply = &list.replies.expect("replies")[0];
    assert_eq!(reply.member.level_info.current_level, 4);
    assert_eq!(reply.member.display_rank, "0");
    assert_eq!(reply.member.is_senior_member, 0);
    assert_eq!(reply.member.vip.vip_status, 1);
    assert_eq!(reply.member.vip.vip_due_date, 1_648_051_200_000);
    assert_eq!(reply.member.vip.theme_type, 0);
    assert_eq!(reply.content.max_line, 6);
    assert!(reply.content.jump_url.is_object());
    assert_eq!(reply.reply_control.time_desc, "6小时前发布");
    assert!(list.config.show_up_flag);
}

#[test]
fn search_type_result_and_video_item_match_api() {
    let root = r##"{
        "seid": "1",
        "page": 1,
        "pagesize": 20,
        "numResults": 1000,
        "numPages": 50,
        "suggest_keyword": "",
        "rqt_type": "search",
        "cost_time": {},
        "exp_list": {},
        "egg_hit": 0,
        "pageinfo": {},
        "result": [],
        "show_column": 0
    }"##;
    let result: SearchTypeResult = serde_json::from_str(root).expect("SearchTypeResult");
    assert_eq!(result.pagesize, 20);
    assert_eq!(result.num_results, 1000);
    assert_eq!(result.num_pages, 50);

    let item = r##"{
        "type": "video",
        "id": 78977417,
        "author": "MitchieM",
        "mid": 5669526,
        "typeid": "30",
        "typename": "VOCALOID·UTAU",
        "arcurl": "http://www.bilibili.com/video/av78977417",
        "aid": 78977417,
        "bvid": "BV1KJ411C7Un",
        "title": "标题",
        "description": "简介",
        "arcrank": "0",
        "pic": "cover",
        "play": 2915520,
        "video_review": 14572,
        "favorites": 114102,
        "tag": "tag",
        "review": 6124,
        "pubdate": 1579877678,
        "senddate": 1593099008,
        "duration": "4:2",
        "badgepay": false,
        "hit_columns": ["title", "description", "tag"],
        "view_type": "",
        "is_pay": 0,
        "is_union_video": 0,
        "rank_score": 109020056
    }"##;
    let video: SearchVideo = serde_json::from_str(item).expect("SearchVideo");
    assert_eq!(video.video_review, 14_572);
    assert_eq!(video.hit_columns.len(), 3);
    assert_eq!(video.is_union_video, 0);
    assert_eq!(video.rank_score, 109_020_056);
}

#[test]
fn user_videos_list_is_nested_vlist() {
    let json = r##"{
        "list": {
            "slist": [],
            "tlist": { "1": { "tid": 1, "count": 3, "name": "动画" } },
            "vlist": [{
                "comment": 985,
                "typeid": 250,
                "play": 224185,
                "pic": "cover",
                "subtitle": "",
                "description": "简介",
                "copyright": "1",
                "title": "标题",
                "review": 0,
                "author": "影视飓风",
                "mid": 946974,
                "created": 1745290800,
                "length": "22:11",
                "video_review": 2365,
                "aid": 114375683741573,
                "bvid": "BV1ac5yzhE94",
                "hide_click": false,
                "is_pay": 0,
                "is_union_video": 1
            }]
        },
        "page": { "pn": 1, "ps": 2, "count": 444 }
    }"##;

    let result: UserVideoListResult = serde_json::from_str(json).expect("UserVideoListResult");
    assert_eq!(result.page.pn, 1);
    assert_eq!(result.page.count, 444);
    assert_eq!(result.list.tlist["1"].count, 3);
    let video = &result.list.vlist[0];
    assert_eq!(video.created, 1_745_290_800);
    assert_eq!(video.length, "22:11");
    assert_eq!(video.video_review, 2365);
    assert_eq!(video.is_union_video, 1);
}

#[test]
fn user_stat_is_nested() {
    let json = r##"{
        "archive": { "view": 213567370 },
        "article": { "view": 3230808 },
        "likes": 20295095
    }"##;
    let stat: UserStat = serde_json::from_str(json).expect("UserStat");
    assert_eq!(stat.archive.view, 213_567_370);
    assert_eq!(stat.article.view, 3_230_808);
    assert_eq!(stat.likes, 20_295_095);
}

#[test]
fn audio_stream_url_matches_api() {
    let json = r##"{
        "sid": 777180,
        "title": "",
        "cover": "",
        "type": 1,
        "info": "",
        "timeout": 10800,
        "size": 5579903,
        "cdns": ["https://a", "https://b"],
        "qualities": null
    }"##;
    let audio: AudioStreamUrl = serde_json::from_str(json).expect("AudioStreamUrl");
    assert_eq!(audio.sid, 777_180);
    assert_eq!(audio.quality, 1);
    assert_eq!(audio.timeout, 10_800);
    assert_eq!(audio.cdns.len(), 2);
    assert!(audio.qualities.is_none());
}

#[test]
fn fav_folder_info_roundtrip_keeps_snake_case() {
    let info: FavFolderInfo = serde_json::from_str(
        r##"{"id":1,"fid":2,"mid":3,"attr":0,"title":"t","media_count":28,"cnt_info":{"collect":1,"play":2,"thumb_up":3,"share":4}}"##,
    )
    .expect("FavFolderInfo");
    assert_eq!(info.media_count, 28);
    let out = serde_json::to_value(&info).unwrap();
    assert_eq!(out["media_count"], 28);
    assert_eq!(out["cnt_info"]["thumb_up"], 3);
    assert!(out.get("mediaCount").is_none());
}

#[test]
fn fav_resource_list_tolerates_null_medias() {
    let data: FavResourceListResult = serde_json::from_str(
        r##"{"info":{"id":1,"fid":2,"mid":3,"attr":0,"title":"t","media_count":0},"medias":null,"has_more":false,"ttl":1}"##,
    )
    .expect("null medias should be treated as empty");
    assert!(data.medias.is_empty());
}

#[test]
fn search_result_unknown_type_is_preserved() {
    let item: SearchResultItem =
        serde_json::from_str(r##"{"type":"activity","id":1,"title":"new type"}"##)
            .expect("unknown result types must not fail the whole response");
    match item {
        SearchResultItem::Other(value) => {
            assert_eq!(value["id"], 1);
            assert_eq!(value["title"], "new type");
        }
        other => panic!("expected Other, got {other:?}"),
    }

    let known: SearchResultItem = serde_json::from_str(
        r##"{"type":"video","id":1,"bvid":"BV1","title":"t","video_review":5}"##,
    )
    .expect("known type");
    assert!(matches!(known, SearchResultItem::Video(_)));
}

#[test]
fn related_items_use_live_response_shape() {
    let items: Vec<VideoRecommendItem> = serde_json::from_str(
        r##"[{
            "aid": 681309864,
            "videos": 1,
            "tid": 21,
            "tname": "日常",
            "pic": "cover",
            "title": "标题",
            "pubdate": 1745000000,
            "duration": 213,
            "cid": 123,
            "bvid": "BV1GJ411x7h7",
            "owner": {"mid": 1, "name": "UP", "face": "face"},
            "stat": {"view": 100, "danmaku": 2, "reply": 3, "favorite": 4, "coin": 5, "share": 6, "like": 7}
        }]"##,
    )
    .expect("related data is an array of video items");
    assert_eq!(items[0].duration, 213);
    assert_eq!(items[0].owner.as_ref().unwrap().name, "UP");
    assert_eq!(items[0].stat.as_ref().unwrap().view, 100);
}
