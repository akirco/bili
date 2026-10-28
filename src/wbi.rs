use crate::client::Params;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

const DEFAULT_TTL: Duration = Duration::from_hours(6);

/// Cached WBI keys, including the precomputed `mixin_key` so it does not have
/// to be rebuilt for every signed request.
#[derive(Clone)]
pub struct WbiCache {
    pub img_key: String,
    pub sub_key: String,
    pub mixin_key: String,
    pub expires: Instant,
}

impl WbiCache {
    #[must_use]
    pub fn new(img_key: String, sub_key: String) -> Self {
        Self::with_ttl(img_key, sub_key, DEFAULT_TTL)
    }

    #[must_use]
    pub fn with_ttl(img_key: String, sub_key: String, ttl: Duration) -> Self {
        let mut mixin_key = String::with_capacity(32);
        let raw = format!("{img_key}{sub_key}");
        for &i in MIXIN_KEY_ENC_TAB.iter().take(32) {
            if let Some(&byte) = raw.as_bytes().get(i) {
                mixin_key.push(byte as char);
            }
        }
        Self {
            img_key,
            sub_key,
            mixin_key,
            expires: Instant::now() + ttl,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        Instant::now() < self.expires
    }
}

/// Shuffle `img_key` + `sub_key` with [`MIXIN_KEY_ENC_TAB`] and take the first 32 chars.
#[must_use]
pub fn get_mixin_key(orig: &str) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .filter_map(|&i| orig.as_bytes().get(i))
        .map(|&b| b as char)
        .collect()
}

/// Sign `params`, returning them with `wts` and `w_rid` appended.
///
/// The returned params have `!'()*` stripped from values, matching what is sent
/// on the wire and what `w_rid` is computed over.
#[must_use]
#[allow(clippy::needless_pass_by_value)] // by-value keeps the public API chainable
pub fn signed_params(params: Params, img_key: &str, sub_key: &str) -> Params {
    let mixin_key = get_mixin_key(&format!("{img_key}{sub_key}"));
    signed_params_with_mixin(&params, &mixin_key)
}

/// Sign `params` using a precomputed `mixin_key`.
pub(crate) fn signed_params_with_mixin(params: &Params, mixin_key: &str) -> Params {
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    signed_params_at(params, mixin_key, wts)
}

fn signed_params_at(params: &Params, mixin_key: &str, wts: u64) -> Params {
    // B 站要求「参与签名的值」和「实际发送的值」一致：先把 `!'()*` 过滤掉，
    // 再对过滤后的参数排序、签名。否则关键词里出现这些字符时 w_rid 会校验失败。
    let filtered: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (k.to_string(), filter_sign_chars(v)))
        .collect();
    let mut params = Params::from(filtered);

    params.insert("wts".to_string(), wts.to_string());
    params.sort_by_key();

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let w_rid = format!("{:x}", md5::compute(format!("{query}{mixin_key}")));
    params.insert("w_rid".to_string(), w_rid);
    params
}

/// 过滤 WBI 签名要求剔除的字符（`!'()*`）。
fn filter_sign_chars(value: &str) -> String {
    value
        .chars()
        .filter(|c| !matches!(c, '!' | '\'' | '(' | ')' | '*'))
        .collect()
}

/// `encodeURIComponent`-compatible percent-encoding.
///
/// Keeps `A-Z a-z 0-9 - _ . ! ~ * ' ( )` as-is, encodes every other byte as
/// uppercase `%XX` (in particular space becomes `%20`, never `+`).
fn percent_encode(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => result.push(byte as char),
            _ => {
                result.push('%');
                result.push(HEX[(byte >> 4) as usize] as char);
                result.push(HEX[(byte & 0x0F) as usize] as char);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_mixin_key_length() {
        // Input must have characters at all indices used by MIXIN_KEY_ENC_TAB (first 32 entries)
        let orig = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ01";
        let key = get_mixin_key(orig);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn get_mixin_key_matches_reference_vector() {
        // From docs/misc/sign/wbi.md
        let key = get_mixin_key("7cd084941338484aae1ad9425b84077c4932caff0ff746eab6f01bf08b70ac45");
        assert_eq!(key, "ea1db124af3c7062474693fa704f4ff8");
    }

    #[test]
    fn signed_params_matches_reference_vector() {
        let params = Params::new()
            .push("foo", "114")
            .push("bar", "514")
            .push("zab", "1919810");
        let signed = signed_params_at(&params, "ea1db124af3c7062474693fa704f4ff8", 1_702_204_169);
        let w_rid = signed
            .iter()
            .find(|(k, _)| *k == "w_rid")
            .map(|(_, v)| v.to_string());
        assert_eq!(w_rid.as_deref(), Some("8f6f2b5b3d485fe1886cec6a0be8c5d4"));
    }

    #[test]
    fn percent_encode_space_and_specials() {
        assert_eq!(percent_encode("one one four"), "one%20one%20four");
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(percent_encode("五一四"), "%E4%BA%94%E4%B8%80%E5%9B%9B");
        // `!'()*` are stripped from values before signing and sending
        assert_eq!(filter_sign_chars("a!b'c(d)e*f"), "abcdef");
        assert_eq!(percent_encode(&filter_sign_chars("a!b'c(d)e*f")), "abcdef");
    }

    #[test]
    fn signed_params_sends_the_filtered_values_it_signs() {
        let params = Params::new().push("keyword", "don't (rust)*!");
        let signed = signed_params(params, "img_key", "sub_key");
        let value = signed
            .iter()
            .find(|(k, _)| *k == "keyword")
            .map(|(_, v)| v.to_string());
        assert_eq!(value.as_deref(), Some("dont rust"));
    }

    #[test]
    fn signed_params_contains_wts_and_w_rid() {
        let params = Params::new().push("foo", "bar");
        let result = signed_params(params, "img_key", "sub_key");
        assert!(result.iter().any(|(k, _)| k == "wts"));
        assert!(result.iter().any(|(k, _)| k == "w_rid"));
        assert!(result.iter().any(|(k, v)| k == "foo" && v == "bar"));
    }

    #[test]
    fn signed_params_deterministic_for_same_input() {
        let p1 = Params::new().push("key", "value");
        let r1 = signed_params(p1, "img_key", "sub_key");

        let rid = r1
            .iter()
            .find(|(k, _)| *k == "w_rid")
            .unwrap()
            .1
            .to_string();
        assert_eq!(rid.len(), 32);
        assert!(rid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn wbi_cache_validity() {
        let cache = WbiCache::with_ttl(
            "img".to_string(),
            "sub".to_string(),
            Duration::from_secs(3600),
        );
        assert!(cache.is_valid());

        let expired =
            WbiCache::with_ttl("img".to_string(), "sub".to_string(), Duration::from_secs(0));
        std::thread::sleep(Duration::from_millis(10));
        assert!(!expired.is_valid());
    }
}
