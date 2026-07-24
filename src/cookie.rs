use reqwest::header::HeaderValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SAVE_INTERVAL: Duration = Duration::from_secs(10);

/// Bilibili cookie store — persists auth cookies to disk,
/// auto-exports Set-Cookie from responses.
pub struct CookieStore {
    cookies: HashMap<String, String>,
    path: PathBuf,
    csrf: String,
    last_save: Instant,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Persisted {
    cookies: HashMap<String, String>,
}

impl CookieStore {
    pub fn new(path: PathBuf) -> Self {
        let persisted: Persisted = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(Persisted {
                cookies: HashMap::new(),
            });

        let csrf = persisted
            .cookies
            .get("bili_jct")
            .cloned()
            .unwrap_or_default();

        Self {
            cookies: persisted.cookies,
            path,
            csrf,
            last_save: Instant::now(),
        }
    }

    /// Whether the user is logged in (has SESSDATA cookie).
    pub fn is_logged_in(&self) -> bool {
        self.cookies.contains_key("SESSDATA")
    }

    /// Check if a cookie key exists.
    pub fn has(&self, key: &str) -> bool {
        self.cookies.contains_key(key)
    }

    /// Get a cookie value by key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cookies.get(key).map(|s| s.as_str())
    }

    /// CSRF token (from bili_jct cookie).
    pub fn csrf_token(&self) -> &str {
        &self.csrf
    }

    /// Build Cookie header value for requests.
    pub fn build_cookie_header(&self) -> Option<HeaderValue> {
        let parts: Vec<String> = self
            .cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        if parts.is_empty() {
            return None;
        }

        let header = HeaderValue::from_str(&parts.join("; ")).ok()?;
        Some(header)
    }

    /// Get SESSDATA value.
    pub fn get_sessdata(&self) -> Option<&str> {
        self.cookies.get("SESSDATA").map(|s| s.as_str())
    }

    /// Parse cookies from a URL's query parameters.
    pub fn set_from_url(&mut self, url: &reqwest::Url) {
        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "SESSDATA" => {
                    self.cookies.insert("SESSDATA".to_string(), v.into_owned());
                }
                "bili_jct" => {
                    self.cookies.insert("bili_jct".to_string(), v.into_owned());
                }
                "DedeUserID" => {
                    self.cookies
                        .insert("DedeUserID".to_string(), v.into_owned());
                }
                _ => {}
            }
        }
        // Update csrf from bili_jct
        if let Some(jct) = self.cookies.get("bili_jct") {
            self.csrf = jct.clone();
        }
        self.flush_if_stale();
    }

    /// Extract Set-Cookie from response headers and update internal state.
    pub fn update_from_response(&mut self, headers: &reqwest::header::HeaderMap) {
        let mut changed = false;

        for set_cookie in headers.get_all("set-cookie").iter() {
            let Ok(val) = set_cookie.to_str() else {
                continue;
            };
            let Some(cookie_part) = val.split(';').next() else {
                continue;
            };
            let Some((name, value)) = cookie_part.split_once('=') else {
                continue;
            };
            let name = name.trim().to_string();
            let value = value.trim().to_string();

            if !value.is_empty() && self.cookies.get(&name) != Some(&value) {
                self.cookies.insert(name, value);
                changed = true;
            }
        }

        // Update csrf from bili_jct
        if let Some(jct) = self.cookies.get("bili_jct")
            && self.csrf != *jct
        {
            self.csrf = jct.clone();
            changed = true;
        }

        if changed {
            self.flush_if_stale();
        }
    }

    /// Force flush cookies to disk.
    pub fn flush(&self) {
        let _ = persist_to_disk(&self.path, &self.cookies);
        // Note: last_save is behind &self, so we accept the minor imprecision
        // of not updating it here. This is only called from flush_if_stale
        // and Drop, where the timing window is acceptable.
    }

    fn flush_if_stale(&mut self) {
        if self.last_save.elapsed() >= SAVE_INTERVAL {
            let _ = persist_to_disk(&self.path, &self.cookies);
            self.last_save = Instant::now();
        }
    }
}

fn persist_to_disk(path: &PathBuf, cookies: &HashMap<String, String>) -> std::io::Result<()> {
    let persisted = Persisted {
        cookies: cookies.clone(),
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&persisted).map_err(|e| std::io::Error::other(e))?;
    std::fs::write(path, &json)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

impl Drop for CookieStore {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Shared cookie store type.
pub type SharedCookieStore = Arc<Mutex<CookieStore>>;

/// Create a new shared cookie store.
pub fn new_shared(path: PathBuf) -> SharedCookieStore {
    Arc::new(Mutex::new(CookieStore::new(path)))
}

/// Default cookie path: ~/.config/bili/cookies.json
pub fn default_cookie_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("bili")
        .join("cookies.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cookie_path() -> PathBuf {
        let dir = std::env::temp_dir().join("bili_cookie_test");
        let _ = std::fs::create_dir_all(&dir);
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.join(format!("test_{}.json", ts))
    }

    #[test]
    fn new_store_creates_empty() {
        let path = temp_cookie_path();
        let store = CookieStore::new(path.clone());
        assert!(!store.is_logged_in());
        assert_eq!(store.csrf_token(), "");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn new_store_loads_existing() {
        let path = temp_cookie_path();
        let mut cookies = HashMap::new();
        cookies.insert("SESSDATA".to_string(), "test_token".to_string());
        cookies.insert("bili_jct".to_string(), "csrf123".to_string());
        let persisted = Persisted { cookies };
        let json = serde_json::to_string_pretty(&persisted).unwrap();
        std::fs::write(&path, &json).unwrap();

        let store = CookieStore::new(path.clone());
        assert!(store.is_logged_in());
        assert_eq!(store.csrf_token(), "csrf123");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn flush_persists_cookies() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        store.cookies.insert("key".to_string(), "value".to_string());
        store.flush();

        let content = std::fs::read_to_string(&path).unwrap();
        let loaded: Persisted = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.cookies.get("key").unwrap(), "value");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_cookie_header_contains_cookies() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        store
            .cookies
            .insert("SESSDATA".to_string(), "abc".to_string());
        store
            .cookies
            .insert("bili_jct".to_string(), "xyz".to_string());
        let header = store.build_cookie_header().unwrap();
        let val = header.to_str().unwrap();
        assert!(val.contains("SESSDATA=abc"));
        assert!(val.contains("bili_jct=xyz"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_cookie_header_empty_returns_none() {
        let path = temp_cookie_path();
        let store = CookieStore::new(path.clone());
        assert!(store.build_cookie_header().is_none());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn set_from_url_parses_sessdata() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        let url = reqwest::Url::parse(
            "https://bilibili.com/?SESSDATA=test123&bili_jct=csrf456&DedeUserID=789",
        )
        .unwrap();
        store.set_from_url(&url);
        assert!(store.is_logged_in());
        assert_eq!(store.csrf_token(), "csrf456");
        assert_eq!(store.get_sessdata(), Some("test123"));
        let _ = std::fs::remove_file(path);
    }
}
