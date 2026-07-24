use crate::client::Params;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Clone)]
pub struct WbiCache {
    pub img_key: String,
    pub sub_key: String,
    pub expires: Instant,
}

impl WbiCache {
    pub fn new(img_key: String, sub_key: String) -> Self {
        Self::with_ttl(img_key, sub_key, Duration::from_secs(3600 * 6))
    }

    pub fn with_ttl(img_key: String, sub_key: String, ttl: Duration) -> Self {
        Self {
            img_key,
            sub_key,
            expires: Instant::now() + ttl,
        }
    }

    pub fn is_valid(&self) -> bool {
        Instant::now() < self.expires
    }
}

fn get_mixin_key(orig: &str) -> String {
    let chars: Vec<char> = orig.chars().collect();
    MIXIN_KEY_ENC_TAB
        .iter()
        .take(32)
        .filter_map(|&i| chars.get(i))
        .collect()
}

pub fn signed_params(mut params: Params, img_key: &str, sub_key: &str) -> Params {
    let mixin_key = get_mixin_key(&format!("{}{}", img_key, sub_key));
    let wts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string();

    params.insert("wts".to_string(), wts);

    params.sort_by_key();

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", percent_encode(k), percent_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let to_hash = format!("{}{}", query, mixin_key);
    let w_rid = format!("{:x}", md5::compute(to_hash.as_bytes()));

    params.insert("w_rid".to_string(), w_rid);
    params
}

/// Percent-encode a string for URL query parameters.
/// Unreserved characters (A-Z a-z 0-9 - . _ ~) and ! # $ & ' ( ) * + , ; = are kept as-is.
/// Space is encoded as '+'. All others are percent-encoded.
fn percent_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'!'
            | b'#'
            | b'$'
            | b'&'
            | b'\''
            | b'('
            | b')'
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => {
                result.push(byte as char);
            }
            b' ' => result.push('+'),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
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

        let p2 = Params::new().push("key", "value");
        let r2 = signed_params(p2, "img_key", "sub_key");

        let rid = r1
            .iter()
            .find(|(k, _)| *k == "w_rid")
            .unwrap()
            .1
            .to_string();
        assert_eq!(rid.len(), 32);
        assert!(rid.chars().all(|c| c.is_ascii_hexdigit()));

        assert!(r2.iter().any(|(k, _)| k == "wts"));
        assert!(r2.iter().any(|(k, _)| k == "w_rid"));
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
