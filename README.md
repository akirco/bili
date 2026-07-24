# bili


## 安装
```
cargo install --git https://github.com/akirco/bili.git
```


## 使用

```bash
Bilibili API CLI

Usage: bili [OPTIONS] [COMMAND]

Commands:
  info              Get video info [requires: --aid|--bvid]
  popular           Popular videos [options: --pn|--ps]
  related           Related recommendations [requires: --aid|--bvid]
  rcmd              Homepage recommendations [options: --ps]
  hot               Ranking by region [options: --pn|--ps|--rid]
  user              User profile [requires: --mid]
  mine              Get current logged-in users own info
  user-videos       User submitted videos [requires: --mid]
  user-stat         User stats [requires: --mid]
  followings        Following list [requires: --mid]
  follow            Follow a user [requires: --fid]
  unfollow          Unfollow a user [requires: --fid]
  search            Search [requires: --keyword]
  hot-search        Hot search words [options: --limit]
  comments          Comment list [requires: --oid]
  add-comment       Post a comment [requires: --oid, --message]
  fav-list          Favorite folder list [requires: --mid]
  fav-items         Favorite folder contents [requires: --fid]
  has-favorite      Check if video is favorited [requires: --aid]
  danmaku           Danmaku list [requires: --cid]
  send-danmaku      Send danmaku [requires: --oid, --message]
  history           Watch history [options: --pn|--ps]
  toview            Watch later list
  toview-add        Add video to watch later [requires: --aid|--bvid]
  login             Login via QR code
  like              Like a video [requires: --aid]
  coin              Coin a video [requires: --aid]
  triple            Triple like+coin+fav [requires: --aid]
  subtitle          Subtitle list [requires: --aid, --cid]
  subtitle-summary  Subtitle summary [requires: --aid, --cid]
  audio             Audio stream URL [requires: --sid]
  playurl           Video play URL [requires: --aid|--bvid, --cid]
  download          Download video [requires: --bvid]
  help              Print this message or the help of the given subcommand(s)

Options:
      --cookies <COOKIES>    Cookie string (SESSDATA=xx; bili_jct=xx)
      --raw                  Output raw JSON (single line)
      --compact              Output data field only
      --completions <SHELL>  Generate shell completions (bash/zsh/fish/powershell/elvish)
  -h, --help                 Print help
```

## License

MIT

## 参考

[BAC](https://github.com/pskdje/bilibili-API-collect/)
