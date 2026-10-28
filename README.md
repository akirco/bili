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
  info              Get video info [requires: --bvid]
  popular           Popular videos [options: --pn|--ps]
  related           Related recommendations [requires: --bvid]
  rcmd              Homepage recommendations [options: --ps]
  hot               Ranking by region [options: --pn|--ps|--rid]
  user              User profile [requires: --mid]
  mine              Get current logged-in users own info
  user-videos       User submitted videos [requires: --mid]
  user-stat         User stats [requires: --mid]
  followings        Following list [options: --mid]
  follow            Follow a user [requires: --fid]
  unfollow          Unfollow a user [requires: --fid]
  user-batch-follow Batch follow users [requires: --fids]
  user-batch-block  Batch block users [requires: --fids]
  search            Search [requires: --keyword]
  hot-search        Hot search words [options: --limit]
  comments          Comment list [requires: --oid]
  add-comment       Post a comment [requires: --oid, --message]
  comment-like      Like/unlike a comment [requires: --oid, --rpid]
  comment-hate      Dislike/undo a comment [requires: --oid, --rpid]
  comment-delete    Delete a comment [requires: --oid, --rpid]
  comment-top       Pin/unpin a comment [requires: --oid, --rpid]
  comment-report    Report a comment [requires: --oid, --rpid, --reason]
  fav-list          Favorite folder list [options: --mid]
  fav-items         Favorite folder contents [requires: --media-id]
  has-favorite      Check if video is favorited [requires: --bvid]
  fav-create        Create a favorite folder [requires: --title]
  fav-edit          Edit a favorite folder [requires: --media-id, --title]
  fav-delete        Delete favorite folders [requires: --media-ids]
  fav-add           Add a video to a favorite folder [requires: --bvid, --media-id]
  fav-remove        Remove a video from a favorite folder [requires: --bvid, --media-id]
  fav-clean         Remove all videos from a favorite folder [requires: --media-id]
  danmaku           Danmaku list [requires: --bvid]
  send-danmaku      Send danmaku [requires: --bvid, --message]
  danmaku-recall    Recall a danmaku [requires: --dmid, --cid]
  history           Watch history [options: --max|--view-at|--business|--ps]
  history-del       Delete one watch-history record [requires: --aid]
  history-clear     Clear all watch history
  toview            Watch later list
  toview-add        Add video to watch later [requires: --bvid]
  toview-del        Remove a video from watch later [options: --aid|--viewed]
  toview-clear      Clear watch later list
  login             Login via QR code
  like              Like a video [requires: --bvid]
  coin              Coin a video [requires: --bvid]
  triple            Triple like+coin+fav [requires: --bvid]
  subtitle          Subtitle list [requires: --bvid]
  subtitle-summary  Subtitle summary [requires: --bvid]
  audio             Audio stream URL [requires: --sid]
  playurl           Video play URL [requires: --bvid]
  download          Download video [options: --dir|--page|--qn|--fourk|--audio-only]
  help              Print this message or the help of the given subcommand(s)

Options:
      --cookies <COOKIES>    Cookie string (SESSDATA=xx; `bili_jct=xx`)
      --raw                  Output raw JSON (single line)
      --compact              Output data field only
      --completions <SHELL>  Generate shell completions (bash/zsh/fish/powershell/elvish)
  -h, --help                 Print help
```

> 所有视频相关命令统一使用 `--bvid`，`aid`/`cid` 由程序自动解析；多P 视频可用 `--page` 指定分P（默认 P1）。

> `download` 会下载 DASH 音视频流并用 `ffmpeg` 合并为 mp4，请先安装 `ffmpeg`；`--dir` 不存在时会自动创建，`--page` 可只下载某个分P，`--qn` 指定清晰度（80=1080P、120=4K 等），`--fourk` 允许 4K，`--audio-only` 只下载 `.m4a` 音轨。大文件下载使用独立的无总超时客户端，并支持基于 `Range` 的断点续传；中断后重新执行同一命令即可继续。

> `fav-list` 返回的 `id` 才是 `fav-items --media-id` 需要的收藏夹 media_id；`fid` 是旧版短 id，不要混用。

> `audio` 是 B 站历史音频接口（音频区已停止投稿），目前仍可播放历史音频，付费音频仅返回试听片段。

## License

MIT

## 参考

[BAC](https://github.com/pskdje/bilibili-API-collect/)
