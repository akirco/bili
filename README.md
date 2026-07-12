# bili


## 安装
```
cargo install --git https://github.com/akirco/bili.git
```


## 使用

```bash
Bilibili API CLI

Usage: bili [OPTIONS]

Options:
  -h, --help  Print help

Actions:
      --info                         Get video info [requires: --aid|--bvid]
      --popular                      Popular videos
      --related                      Related recommendations [requires: --aid|--bvid]
      --rcmd                         Homepage recommendations
      --hot                          Ranking by region
      --user                         User profile [requires: --mid]
      --user-videos                  User submitted videos [requires: --mid]
      --user-stat                    User stats [requires: --mid]
      --followings                   Following list [requires: --mid]
      --follow                       Follow a user [requires: --fid]
      --unfollow                     Unfollow a user [requires: --fid]
      --search <SEARCH>              Search (value=keyword)
      --hot-search                   Hot search words
      --comments                     Comment list [requires: --oid]
      --add-comment <ADD_COMMENT>    Post a comment [requires: --oid, value=message]
      --fav-list                     Favorite folder list [requires: --mid]
      --has-favorite                 Check if video is favorited [requires: --aid]
      --danmaku                      Danmaku list [requires: --cid]
      --send-danmaku <SEND_DANMAKU>  Send danmaku [requires: --oid, value=message]
      --history                      Watch history
      --toview                       Watch later list
      --toview-add                   Add video to watch later [requires: --aid|--bvid]
      --login                        Login via QR code
      --like                         Like a video [requires: --aid]
      --coin                         Coin a video [requires: --aid]
      --triple                       Triple like+coin+fav [requires: --aid]
      --subtitle                     Subtitle list [requires: --aid, --cid]
      --subtitle-summary             Subtitle summary [requires: --aid, --cid]
      --audio                        Audio stream URL [requires: --sid]
      --playurl                      Video play URL [requires: --aid|--bvid, --cid]

IDs:
      --aid <AID>    Video avid
      --bvid <BVID>  Video bvid
      --cid <CID>    Video cid
      --mid <MID>    User mid
      --fid <FID>    Target user mid (follow/unfollow)
      --oid <OID>    Object id (comment oid / danmaku cid)
      --sid <SID>    Audio sid

Options:
      --type <TYPE>          Type (comment: 1=video; search: video/bili_user/...)
      --sort <SORT>          Sort (comment: 0=time,2=like,3=reply)
      --mode <MODE>          Mode (danmaku: 1=normal,4=bottom,5=top)
      --multiply <MULTIPLY>  Coin count (1 or 2)
      --progress <PROGRESS>  Playback position in ms (danmaku)
      --rid <RID>            Region id (ranking)
      --page <PAGE>          Page number
      --pn <PN>              Page number
      --ps <PS>              Page size
      --limit <LIMIT>        Limit / max segments (danmaku)
      --qn <QN>              Quality (16=360P..127=8K)
      --fnval <FNVAL>        Format (1=MP4,16=DASH,4095=all)
      --fourk <FOURK>        Allow 4K (0=no,1=yes)
      --segment <SEGMENT>    Segment index (subtitle summary)

Global:
      --cookies <COOKIES>
          Cookie string (SESSDATA=xx; bili_jct=xx)
      --import-cookies <IMPORT_COOKIES>
          Import cookies from file
      --export-cookies <EXPORT_COOKIES>
          Export cookies to file
      --raw
          Output raw JSON (single line)
      --compact
          Output data field only
      --completions <SHELL>
          Generate shell completions (bash/zsh/fish/powershell/elvish)
```

## License

MIT

## 参考

[BAC](https://github.com/pskdje/bilibili-API-collect/)
