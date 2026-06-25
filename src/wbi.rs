use std::collections::HashMap;
use std::time::{Duration, Instant};

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
        Self {
            img_key,
            sub_key,
            expires: Instant::now() + Duration::from_secs(3600 * 6),
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

pub fn signed_params(
    mut params: HashMap<String, String>,
    img_key: &str,
    sub_key: &str,
) -> HashMap<String, String> {
    let mixin_key = get_mixin_key(&format!("{}{}", img_key, sub_key));
    let wts = chrono::Utc::now().timestamp().to_string();

    params.insert("wts".to_string(), wts);

    let mut sorted: Vec<_> = params.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let query: String = {
        let mut ser = url::form_urlencoded::Serializer::new(String::new());
        for (k, v) in &sorted {
            ser.append_pair(k, v);
        }
        ser.finish()
    };

    let to_hash = format!("{}{}", query, mixin_key);
    let w_rid = format!("{:x}", md5::compute(to_hash.as_bytes()));

    params.insert("w_rid".to_string(), w_rid);
    params
}
