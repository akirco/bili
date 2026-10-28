use crate::client::{BiliClient, BiliClientInner, BiliResponse, FormBuilder, Params};
use crate::error::BiliError;
use crate::types::{DanmakuElem, DanmakuHistoryResult, DanmakuSendResult, DanmakuSnapshotResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct DanmakuClient {
    client: BiliClient,
}

impl DanmakuClient {
    pub(crate) fn new(inner: Arc<BiliClientInner>) -> Self {
        Self {
            client: BiliClient { inner },
        }
    }

    /// Fetch danmaku for a video using the protobuf `seg.so` API.
    ///
    /// `cid` is the content/part ID. `segment_index` starts at 1 (each segment covers ~6 minutes).
    /// Returns parsed `DanmakuElem` entries.
    pub async fn seg(&self, cid: i64, segment_index: i64) -> Result<Vec<DanmakuElem>, BiliError> {
        let url = format!(
            "https://api.bilibili.com/x/v2/dm/web/seg.so?type=1&oid={cid}&segment_index={segment_index}"
        );
        let bytes = self.client.get_bytes(&url).await?;
        Ok(parse_seg_so(&bytes))
    }

    /// Fetch all danmaku segments for a video (segments 1 through `max_segments`).
    ///
    /// Each segment covers roughly 6 minutes of video. Default `max_segments` is 10
    /// (covers ~60 minutes). Adjust for longer videos.
    pub async fn all_segments(
        &self,
        cid: i64,
        max_segments: i64,
    ) -> Result<Vec<DanmakuElem>, BiliError> {
        /// Segments fetched concurrently per batch.
        const BATCH: i64 = 4;

        let mut all = Vec::new();
        let mut start = 1;
        while start <= max_segments {
            let end = (start + BATCH - 1).min(max_segments);

            let mut set = tokio::task::JoinSet::new();
            for seg in start..=end {
                let client = self.clone();
                set.spawn(async move { (seg, client.seg(cid, seg).await) });
            }

            let mut by_seg: std::collections::BTreeMap<i64, Vec<DanmakuElem>> =
                std::collections::BTreeMap::new();
            let mut first_error = None;
            while let Some(joined) = set.join_next().await {
                match joined {
                    Ok((seg, Ok(elems))) => {
                        by_seg.insert(seg, elems);
                    }
                    Ok((_, Err(e))) => {
                        first_error.get_or_insert(e);
                    }
                    Err(e) => {
                        first_error.get_or_insert_with(|| {
                            BiliError::Parse(format!("danmaku task failed: {e}"))
                        });
                    }
                }
            }
            if let Some(e) = first_error {
                return Err(e);
            }

            let mut hit_empty_segment = false;
            for seg in start..=end {
                match by_seg.remove(&seg) {
                    Some(elems) if elems.is_empty() => {
                        // Don't discard non-empty later segments in this batch:
                        // an empty segment can be a gap, not necessarily the end.
                        hit_empty_segment = true;
                    }
                    Some(elems) => all.extend(elems),
                    None => {}
                }
            }
            if hit_empty_segment {
                break;
            }
            start = end + 1;
            if start <= max_segments {
                // Be gentle with Bilibili's rate limiting on long videos.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
        Ok(all)
    }

    /// Legacy: fetch danmaku snapshot (old JSON API, kept for backward compatibility).
    pub async fn snapshot(
        &self,
        aid: i64,
    ) -> Result<BiliResponse<DanmakuSnapshotResult>, BiliError> {
        let params = Params::new().push_val("aid", aid);
        self.client
            .get("https://api.bilibili.com/x/v2/dm/ajax", params)
            .await
    }

    /// Fetch history danmaku dates for a given month.
    pub async fn history_dates(
        &self,
        oid: i64,
        month: &str,
    ) -> Result<BiliResponse<DanmakuHistoryResult>, BiliError> {
        let params = Params::new()
            .push("type", "1")
            .push_val("oid", oid)
            .push("month", month);
        self.client
            .get("https://api.bilibili.com/x/v2/dm/history/index", params)
            .await
    }

    /// Send a video danmaku.
    ///
    /// Requires login (SESSDATA + CSRF). `oid` is the video cid.
    /// `mode`: 1=normal, 4=bottom, 5=top.
    /// Provide either `aid` or `bvid`.
    pub async fn send(
        &self,
        oid: i64,
        msg: &str,
        mode: i32,
        progress: i64,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> Result<BiliResponse<DanmakuSendResult>, BiliError> {
        let csrf = self.client.csrf()?;

        let mut form = FormBuilder::new()
            .push("type", "1")
            .push_val("oid", oid)
            .push("msg", msg)
            .push_val("mode", mode)
            .push_val("progress", progress)
            .csrf(csrf.clone());

        if let Some(aid) = aid {
            form = form.push_val("aid", aid);
        }
        if let Some(bvid) = bvid {
            form = form.push("bvid", bvid);
        }

        let wbi_params = Params::new()
            .push("csrf", csrf)
            .push("web_location", "1315873");

        self.client
            .wbi_post::<DanmakuSendResult>(
                "https://api.bilibili.com/x/v2/dm/post",
                wbi_params,
                &form.build(),
            )
            .await
    }

    /// Recall a danmaku within 2 minutes (limited to 3 recalls per day).
    pub async fn recall(
        &self,
        dmid: i64,
        cid: i64,
    ) -> Result<BiliResponse<serde_json::Value>, BiliError> {
        let csrf = self.client.csrf()?;

        let form = FormBuilder::new()
            .push_val("dmid", dmid)
            .push_val("cid", cid)
            .csrf(csrf)
            .build();

        self.client
            .post("https://api.bilibili.com/x/dm/recall", &form)
            .await
    }
}

// --- Hand-rolled protobuf parser (no prost dependency) ---
// Based on bilidanm's parser, adapted for the library.

fn decode_varint(data: &[u8], offset: &mut usize) -> Option<u64> {
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        if *offset >= data.len() {
            return None;
        }
        let byte = data[*offset];
        *offset += 1;
        result |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    Some(result)
}

/// Decode a varint and convert it to `T`, returning `None` if the value does
/// not fit in `T`.
fn cast_varint<T: TryFrom<u64>>(data: &[u8], offset: &mut usize) -> Option<T> {
    T::try_from(decode_varint(data, offset)?).ok()
}

fn skip_field(wire_type: u64, data: &[u8], offset: &mut usize) -> bool {
    match wire_type {
        0 => decode_varint(data, offset).is_some(),
        1 => advance(offset, 8, data.len()),
        2 => {
            let Some(len) = decode_varint(data, offset) else {
                return false;
            };
            let Ok(len) = usize::try_from(len) else {
                return false;
            };
            advance(offset, len, data.len())
        }
        5 => advance(offset, 4, data.len()),
        _ => false,
    }
}

/// Advance `*offset` by `len`, returning `false` on overflow or out-of-bounds.
fn advance(offset: &mut usize, len: usize, data_len: usize) -> bool {
    match offset.checked_add(len) {
        Some(end) if end <= data_len => {
            *offset = end;
            true
        }
        _ => false,
    }
}

/// Read a length-delimited UTF-8 string, advancing `*offset` to the end of the
/// field. Returns `None` on truncated or out-of-range input.
fn read_string(data: &[u8], offset: &mut usize) -> Option<String> {
    let len = decode_varint(data, offset)?;
    let len = usize::try_from(len).ok()?;
    let end = offset.checked_add(len)?;
    if end > data.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&data[*offset..end]).to_string();
    *offset = end;
    Some(s)
}

fn parse_seg_so(data: &[u8]) -> Vec<DanmakuElem> {
    let mut elems = Vec::new();
    let mut offset = 0;
    while offset < data.len() {
        let Some(key) = decode_varint(data, &mut offset) else {
            break;
        };
        let wire_type = key & 0x07;
        let field_number = key >> 3;

        if field_number == 1 && wire_type == 2 {
            let Some(len) = decode_varint(data, &mut offset) else {
                break;
            };
            let Ok(len) = usize::try_from(len) else {
                break;
            };
            let Some(end) = offset.checked_add(len) else {
                break;
            };
            if end > data.len() {
                break;
            }
            if let Some(elem) = parse_dm_elem(&data[offset..end]) {
                elems.push(elem);
            }
            offset = end;
        } else if !skip_field(wire_type, data, &mut offset) {
            break;
        }
    }
    elems
}

fn parse_dm_elem(data: &[u8]) -> Option<DanmakuElem> {
    let mut offset = 0;
    let mut id = 0i64;
    let mut progress = 0i64;
    let mut mode = 1i32;
    let mut fontsize = 25i32;
    let mut color = 0x00FF_FFFFu32;
    let mut mid_hash = 0u64;
    let mut content = String::new();
    let mut weight = 0i64;
    let mut action = String::new();
    let mut pool = 0i64;
    let mut id_str = String::new();

    while offset < data.len() {
        let Some(key) = decode_varint(data, &mut offset) else {
            break;
        };
        let wire_type = key & 0x07;
        let field_number = key >> 3;

        match (field_number, wire_type) {
            (1, 0) => id = cast_varint::<i64>(data, &mut offset).unwrap_or(0),
            (2, 0) => progress = cast_varint::<i64>(data, &mut offset).unwrap_or(0),
            (3, 0) => mode = cast_varint::<i32>(data, &mut offset).unwrap_or(1),
            (4, 0) => fontsize = cast_varint::<i32>(data, &mut offset).unwrap_or(25),
            (5, 0) => color = cast_varint::<u32>(data, &mut offset).unwrap_or(0x00FF_FFFF),
            (6, 0) => mid_hash = decode_varint(data, &mut offset).unwrap_or(0),
            (7, 2) => match read_string(data, &mut offset) {
                Some(s) => content = s,
                None => break,
            },
            (9, 0) => weight = cast_varint::<i64>(data, &mut offset).unwrap_or(0),
            (12, 2) => match read_string(data, &mut offset) {
                Some(s) => action = s,
                None => break,
            },
            (13, 0) => pool = cast_varint::<i64>(data, &mut offset).unwrap_or(0),
            (15, 2) => match read_string(data, &mut offset) {
                Some(s) => id_str = s,
                None => break,
            },
            _ => {
                if !skip_field(wire_type, data, &mut offset) {
                    break;
                }
            }
        }
    }

    if content.is_empty() {
        return None;
    }

    Some(DanmakuElem {
        id,
        progress,
        mode,
        fontsize,
        color,
        mid_hash,
        content,
        weight,
        action,
        pool,
        id_str,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_varint_simple() {
        let data = [0x01];
        let mut offset = 0;
        assert_eq!(decode_varint(&data, &mut offset), Some(1));
        assert_eq!(offset, 1);
    }

    #[test]
    fn test_decode_varint_multibyte() {
        // 300 = 0xAC 0x02
        let data = [0xAC, 0x02];
        let mut offset = 0;
        assert_eq!(decode_varint(&data, &mut offset), Some(300));
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_decode_varint_max_uint64() {
        // 0xFFFFFFFFFFFFFFFF = all continuation bits set
        // bytes: FF FF FF FF FF FF FF FF FF 01
        let data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        let mut offset = 0;
        assert_eq!(decode_varint(&data, &mut offset), Some(u64::MAX));
        assert_eq!(offset, 10);
    }

    #[test]
    fn test_decode_varint_overflow() {
        // 11 bytes with continuation bit set → overflow
        let data = [0x80; 11];
        let mut offset = 0;
        assert_eq!(decode_varint(&data, &mut offset), None);
    }

    #[test]
    fn test_decode_varint_empty() {
        let mut offset = 0;
        assert_eq!(decode_varint(&[], &mut offset), None);
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_decode_varint_truncated() {
        // last byte has continuation bit set but no more data
        let data = [0x80, 0x80];
        let mut offset = 0;
        assert_eq!(decode_varint(&data, &mut offset), None);
    }

    #[test]
    fn test_skip_field_wire0() {
        let data = [0xAC, 0x02]; // varint 300
        let mut offset = 0;
        assert!(skip_field(0, &data, &mut offset));
        assert_eq!(offset, 2);
    }

    #[test]
    fn test_skip_field_wire1() {
        let data = [0x00; 8]; // 64-bit, just zeros
        let mut offset = 0;
        assert!(skip_field(1, &data, &mut offset));
        assert_eq!(offset, 8);
    }

    #[test]
    fn test_skip_field_wire2() {
        let data = [0x04, 0x01, 0x02, 0x03, 0x04]; // length 4 + 4 bytes
        let mut offset = 0;
        assert!(skip_field(2, &data, &mut offset));
        assert_eq!(offset, 5);
    }

    #[test]
    fn test_skip_field_wire5() {
        let data = [0x00; 4]; // 32-bit, just zeros
        let mut offset = 0;
        assert!(skip_field(5, &data, &mut offset));
        assert_eq!(offset, 4);
    }

    #[test]
    fn test_skip_field_unknown_wire() {
        let data = [0x00];
        let mut offset = 0;
        assert!(!skip_field(3, &data, &mut offset));
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_parse_dm_elem_minimal() {
        // A minimal danmaku element:
        //   id=1, progress=1000, mode=1, fontsize=25, color=0xFFFFFF,
        //   mid_hash=0, content="test", weight=0, action="", pool=0, id_str=""
        let inner = [
            0x08, 0x01, // field 1: id = 1
            0x10, 0xE8, 0x07, // field 2: progress = 1000
            0x18, 0x01, // field 3: mode = 1
            0x20, 0x19, // field 4: fontsize = 25
            0x28, 0xFF, 0xFF, 0xFF, 0x07, // field 5: color = 0xFFFFFF
            0x30, 0x00, // field 6: mid_hash = 0
            0x3A, 0x04, 0x74, 0x65, 0x73, 0x74, // field 7: content = "test"
            0x48, 0x00, // field 9: weight = 0
            0x62, 0x00, // field 12: action = ""
            0x68, 0x00, // field 13: pool = 0
            0x7A, 0x00, // field 15: id_str = ""
        ];
        let elem = parse_dm_elem(&inner).expect("should parse");
        assert_eq!(elem.id, 1);
        assert_eq!(elem.progress, 1000);
        assert_eq!(elem.mode, 1);
        assert_eq!(elem.fontsize, 25);
        assert_eq!(elem.color, 0x00FF_FFFF);
        assert_eq!(elem.mid_hash, 0);
        assert_eq!(elem.content, "test");
        assert_eq!(elem.weight, 0);
        assert_eq!(elem.action, "");
        assert_eq!(elem.pool, 0);
        assert_eq!(elem.id_str, "");
    }

    #[test]
    fn test_parse_dm_elem_empty_content() {
        // Missing field 7 (content) → should return None
        let inner = [
            0x08, 0x01, // field 1: id = 1
            0x10, 0x01, // field 2: progress = 1
        ];
        assert!(parse_dm_elem(&inner).is_none());
    }

    #[test]
    fn test_parse_seg_so_multiple() {
        // Two danmaku elements
        let inner1 = [
            0x08, 0x01, 0x10, 0x01, 0x3A, 0x03, 0x61, 0x62,
            0x63, // id=1, progress=1, content="abc"
        ];
        let inner2 = [
            0x08, 0x02, 0x10, 0x02, 0x3A, 0x03, 0x64, 0x65,
            0x66, // id=2, progress=2, content="def"
        ];

        let mut data = Vec::new();
        // outer field 1, length-delimited
        data.push(0x0A);
        data.push(u8::try_from(inner1.len()).unwrap());
        data.extend_from_slice(&inner1);
        data.push(0x0A);
        data.push(u8::try_from(inner2.len()).unwrap());
        data.extend_from_slice(&inner2);

        let elems = parse_seg_so(&data);
        assert_eq!(elems.len(), 2);
        assert_eq!(elems[0].id, 1);
        assert_eq!(elems[0].content, "abc");
        assert_eq!(elems[1].id, 2);
        assert_eq!(elems[1].content, "def");
    }

    #[test]
    fn test_parse_seg_so_empty() {
        let elems = parse_seg_so(&[]);
        assert!(elems.is_empty());
    }

    #[test]
    fn test_parse_seg_so_truncated() {
        // incomplete varint at end → should not panic
        let data = [0x0A, 0x03, 0x08, 0x01]; // length=3 but only 2 bytes provided
        let elems = parse_seg_so(&data);
        // truncated inner message is simply skipped
        assert!(elems.is_empty());
    }
}
