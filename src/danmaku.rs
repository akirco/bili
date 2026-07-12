use crate::client::{BiliClient, BiliClientInner, FormBuilder, BiliResponse, Params};
use crate::error::BiliError;
use crate::types::{DanmakuElem, DanmakuSendResult};
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
    pub async fn seg(
        &self,
        cid: i64,
        segment_index: i64,
    ) -> Result<Vec<DanmakuElem>, BiliError> {
        let url = format!(
            "https://api.bilibili.com/x/v2/dm/web/seg.so?type=1&oid={}&segment_index={}",
            cid, segment_index
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
        let mut all = Vec::new();
        for seg in 1..=max_segments {
            let elems = self.seg(cid, seg).await?;
            if elems.is_empty() {
                break;
            }
            all.extend(elems);
        }
        Ok(all)
    }

    /// Legacy: fetch danmaku snapshot (old JSON API, kept for backward compatibility).
    pub async fn snapshot(&self, aid: i64) -> Result<serde_json::Value, BiliError> {
        let params = Params::new().push("aid", aid.to_string());
        self.client
            .get_raw("https://api.bilibili.com/x/v2/dm/ajax", params)
            .await
    }

    /// Fetch history danmaku dates for a given month.
    pub async fn history_dates(
        &self,
        oid: i64,
        month: &str,
    ) -> Result<serde_json::Value, BiliError> {
        let params = Params::new()
            .push("type", "1")
            .push("oid", oid.to_string())
            .push("month", month);
        self.client
            .get_raw("https://api.bilibili.com/x/v2/dm/history/index", params)
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
    ) -> Result<BiliResponse<Option<DanmakuSendResult>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let mut form = FormBuilder::new()
            .push("type", "1")
            .push("oid", oid.to_string())
            .push("msg", msg.to_string())
            .push("mode", mode.to_string())
            .push("progress", progress.to_string())
            .csrf(csrf.clone());

        if let Some(aid) = aid {
            form = form.push("aid", aid.to_string());
        }
        if let Some(bvid) = bvid {
            form = form.push("bvid", bvid.to_string());
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
    pub async fn recall(&self, dmid: i64, cid: i64) -> Result<BiliResponse<Option<serde_json::Value>>, BiliError> {
        let csrf = self.client.csrf().await?;

        let form = FormBuilder::new()
            .push("dmid", dmid.to_string())
            .push("cid", cid.to_string())
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
        result |= ((byte & 0x7F) as u64) << shift;
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

fn skip_field(wire_type: u64, data: &[u8], offset: &mut usize) -> bool {
    match wire_type {
        0 => decode_varint(data, offset).is_some(),
        1 => {
            *offset += 8;
            *offset <= data.len()
        }
        2 => {
            if let Some(len) = decode_varint(data, offset) {
                *offset += len as usize;
                *offset <= data.len()
            } else {
                false
            }
        }
        5 => {
            *offset += 4;
            *offset <= data.len()
        }
        _ => false,
    }
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
            let end = offset + (len as usize);
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
    let mut color = 0xFFFFFFu32;
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
            (1, 0) => id = decode_varint(data, &mut offset).unwrap_or(0) as i64,
            (2, 0) => progress = decode_varint(data, &mut offset).unwrap_or(0) as i64,
            (3, 0) => mode = decode_varint(data, &mut offset).unwrap_or(1) as i32,
            (4, 0) => fontsize = decode_varint(data, &mut offset).unwrap_or(25) as i32,
            (5, 0) => color = decode_varint(data, &mut offset).unwrap_or(0xFFFFFF) as u32,
            (6, 0) => mid_hash = decode_varint(data, &mut offset).unwrap_or(0),
            (7, 2) => {
                let len = decode_varint(data, &mut offset).unwrap_or(0) as usize;
                if offset + len <= data.len() {
                    content = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
                }
                offset += len;
            }
            (9, 0) => weight = decode_varint(data, &mut offset).unwrap_or(0) as i64,
            (12, 2) => {
                let len = decode_varint(data, &mut offset).unwrap_or(0) as usize;
                if offset + len <= data.len() {
                    action = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
                }
                offset += len;
            }
            (13, 0) => pool = decode_varint(data, &mut offset).unwrap_or(0) as i64,
            (15, 2) => {
                let len = decode_varint(data, &mut offset).unwrap_or(0) as usize;
                if offset + len <= data.len() {
                    id_str = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
                }
                offset += len;
            }
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
            0x08, 0x01,                         // field 1: id = 1
            0x10, 0xE8, 0x07,                    // field 2: progress = 1000
            0x18, 0x01,                         // field 3: mode = 1
            0x20, 0x19,                         // field 4: fontsize = 25
            0x28, 0xFF, 0xFF, 0xFF, 0x07,        // field 5: color = 0xFFFFFF
            0x30, 0x00,                         // field 6: mid_hash = 0
            0x3A, 0x04, 0x74, 0x65, 0x73, 0x74, // field 7: content = "test"
            0x48, 0x00,                         // field 9: weight = 0
            0x62, 0x00,                         // field 12: action = ""
            0x68, 0x00,                         // field 13: pool = 0
            0x7A, 0x00,                         // field 15: id_str = ""
        ];
        let elem = parse_dm_elem(&inner).expect("should parse");
        assert_eq!(elem.id, 1);
        assert_eq!(elem.progress, 1000);
        assert_eq!(elem.mode, 1);
        assert_eq!(elem.fontsize, 25);
        assert_eq!(elem.color, 0xFFFFFF);
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
            0x08, 0x01,  // field 1: id = 1
            0x10, 0x01,  // field 2: progress = 1
        ];
        assert!(parse_dm_elem(&inner).is_none());
    }

    #[test]
    fn test_parse_seg_so_multiple() {
        // Two danmaku elements
        let inner1 = [
            0x08, 0x01, 0x10, 0x01, 0x3A, 0x03, 0x61, 0x62, 0x63, // id=1, progress=1, content="abc"
        ];
        let inner2 = [
            0x08, 0x02, 0x10, 0x02, 0x3A, 0x03, 0x64, 0x65, 0x66, // id=2, progress=2, content="def"
        ];

        let mut data = Vec::new();
        // outer field 1, length-delimited
        data.push(0x0A);
        data.push(inner1.len() as u8);
        data.extend_from_slice(&inner1);
        data.push(0x0A);
        data.push(inner2.len() as u8);
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
