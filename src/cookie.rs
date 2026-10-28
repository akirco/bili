use reqwest::header::HeaderValue;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SAVE_INTERVAL: Duration = Duration::from_secs(10);

/// `SameSite` attribute. It is not sent to the server; kept so the cookie jar
/// preserves what the server set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum SameSite {
    Strict,
    Lax,
    None,
}

/// A single cookie with the attributes needed to decide whether it applies to
/// a request (`Domain` / `Path` / expiry / `Secure`).
#[derive(Clone, Debug)]
struct CookieEntry {
    value: String,
    /// Lower-case domain without a leading dot. `None` means a legacy/global
    /// cookie that is sent to every host.
    domain: Option<String>,
    path: Option<String>,
    /// Unix timestamp in seconds. `None` means a session cookie.
    expires: Option<i64>,
    /// `Domain` attribute was absent: only send to the exact host.
    host_only: bool,
    /// Only send over HTTPS.
    secure: bool,
    /// Server-set `SameSite` policy (informational; not part of the Cookie header).
    same_site: Option<SameSite>,
}

impl CookieEntry {
    fn is_expired(&self) -> bool {
        self.expires.is_some_and(|expires| expires <= now_unix())
    }

    fn matches(&self, host: &str, path: &str, is_https: bool) -> bool {
        if self.is_expired() {
            return false;
        }
        if self.secure && !is_https {
            return false;
        }
        let domain_ok = match &self.domain {
            None => true,
            Some(domain) if self.host_only => host.eq_ignore_ascii_case(domain),
            Some(domain) => {
                host.eq_ignore_ascii_case(domain)
                    || host
                        .to_ascii_lowercase()
                        .ends_with(&format!(".{}", domain.to_ascii_lowercase()))
            }
        };
        let path_ok = match self.path.as_deref() {
            None | Some("/") => true,
            Some(cookie_path) => {
                path == cookie_path || path.starts_with(&format!("{cookie_path}/"))
            }
        };
        domain_ok && path_ok
    }
}

/// Bilibili cookie store — persists auth cookies to disk,
/// auto-exports Set-Cookie from responses.
pub struct CookieStore {
    cookies: HashMap<String, CookieEntry>,
    path: PathBuf,
    csrf: String,
    last_save: Instant,
    dirty: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Persisted {
    #[serde(default)]
    cookies: HashMap<String, PersistedCookie>,
}

/// Accept both the old `{"name": "value"}` format and the new
/// `{"name": {"value": ...}}` format.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum PersistedCookie {
    Legacy(String),
    Full {
        value: String,
        #[serde(default)]
        domain: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        expires: Option<i64>,
        #[serde(default)]
        host_only: bool,
        #[serde(default)]
        secure: bool,
        #[serde(default)]
        same_site: Option<SameSite>,
    },
}

impl From<PersistedCookie> for CookieEntry {
    fn from(cookie: PersistedCookie) -> Self {
        match cookie {
            PersistedCookie::Legacy(value) => Self {
                value,
                domain: Some("bilibili.com".to_string()),
                path: Some("/".to_string()),
                expires: None,
                host_only: false,
                secure: false,
                same_site: None,
            },
            PersistedCookie::Full {
                value,
                domain,
                path,
                expires,
                host_only,
                secure,
                same_site,
            } => Self {
                value,
                domain,
                path,
                expires,
                host_only,
                secure,
                same_site,
            },
        }
    }
}

impl PersistedCookie {
    fn from_entry(entry: &CookieEntry) -> Self {
        Self::Full {
            value: entry.value.clone(),
            domain: entry.domain.clone(),
            path: entry.path.clone(),
            expires: entry.expires,
            host_only: entry.host_only,
            secure: entry.secure,
            same_site: entry.same_site,
        }
    }
}

impl CookieStore {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        let persisted = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Persisted>(&s).ok())
            .unwrap_or_default();

        let mut cookies: HashMap<String, CookieEntry> = persisted
            .cookies
            .into_iter()
            .map(|(name, cookie)| (name, cookie.into()))
            .collect();
        let before = cookies.len();
        cookies.retain(|_, cookie| !cookie.is_expired());
        let dirty = cookies.len() != before;

        let csrf = cookies
            .get("bili_jct")
            .map(|cookie| cookie.value.clone())
            .unwrap_or_default();

        Self {
            cookies,
            path,
            csrf,
            last_save: Instant::now(),
            dirty,
        }
    }

    /// Whether the user is logged in (has a non-expired SESSDATA cookie).
    #[must_use]
    pub fn is_logged_in(&self) -> bool {
        self.get("SESSDATA").is_some()
    }

    /// Check if a non-expired cookie key exists.
    #[must_use]
    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    /// Get a non-expired cookie value by key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cookies
            .get(key)
            .filter(|cookie| !cookie.is_expired())
            .map(|cookie| cookie.value.as_str())
    }

    /// Set a cookie bootstrapped at runtime (e.g. `buvid3`).
    pub(crate) fn set_cookie(&mut self, name: &str, value: String) {
        let changed = self.insert(
            name.to_string(),
            CookieEntry {
                value,
                domain: Some("bilibili.com".to_string()),
                path: Some("/".to_string()),
                expires: None,
                host_only: false,
                secure: false,
                same_site: None,
            },
        );
        if changed {
            self.dirty = true;
        }
    }

    /// CSRF token (from `bili_jct` cookie).
    #[must_use]
    pub fn csrf_token(&self) -> &str {
        if self.has("bili_jct") { &self.csrf } else { "" }
    }

    /// Build Cookie header value for requests.
    ///
    /// Keys are emitted in sorted order so the header is stable across calls.
    /// Expired cookies are excluded.
    #[must_use]
    pub fn build_cookie_header(&self) -> Option<HeaderValue> {
        self.header_from(|_| true)
    }

    /// Build the Cookie header for a specific request URL, honoring
    /// `Domain` / `Path` / expiry.
    #[must_use]
    pub fn build_cookie_header_for(&self, url: &reqwest::Url) -> Option<HeaderValue> {
        let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
        let path = url.path();
        let is_https = url.scheme() == "https";
        self.header_from(|cookie| cookie.matches(&host, path, is_https))
    }

    fn header_from<F>(&self, include: F) -> Option<HeaderValue>
    where
        F: Fn(&CookieEntry) -> bool,
    {
        let mut entries: Vec<(&str, &str)> = self
            .cookies
            .iter()
            .filter(|(_, cookie)| !cookie.is_expired() && include(cookie))
            .map(|(name, cookie)| (name.as_str(), cookie.value.as_str()))
            .collect();
        if entries.is_empty() {
            return None;
        }
        entries.sort_by_key(|(name, _)| *name);
        let parts: Vec<String> = entries
            .into_iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        HeaderValue::from_str(&parts.join("; ")).ok()
    }

    /// Get SESSDATA value.
    #[must_use]
    pub fn get_sessdata(&self) -> Option<&str> {
        self.get("SESSDATA")
    }

    /// Parse cookies from a URL's query parameters (used by the QR login flow).
    pub fn set_from_url(&mut self, url: &reqwest::Url) {
        let expires = url
            .query_pairs()
            .find(|(key, _)| key == "Expires")
            .and_then(|(_, value)| value.parse::<i64>().ok());
        self.set_cookies(
            url.query_pairs()
                .map(|(k, v)| (k.into_owned(), v.into_owned())),
            Some("bilibili.com"),
            expires,
            url.scheme() == "https",
        );
    }

    /// Parse a raw `Cookie` header string (`SESSDATA=x; bili_jct=y; DedeUserID=1`).
    pub fn set_from_cookie_string(&mut self, cookies: &str) {
        self.set_cookies(
            cookies.split(';').filter_map(|part| {
                let (k, v) = part.trim().split_once('=')?;
                Some((k.trim().to_string(), v.trim().to_string()))
            }),
            Some("bilibili.com"),
            None,
            false,
        );
    }

    fn set_cookies<I>(
        &mut self,
        cookies: I,
        domain: Option<&str>,
        expires: Option<i64>,
        secure: bool,
    ) where
        I: Iterator<Item = (String, String)>,
    {
        let mut changed = false;
        for (name, value) in cookies {
            match name.as_str() {
                "SESSDATA" | "bili_jct" | "DedeUserID" | "buvid3" | "buvid4" | "b_nut" => {
                    let entry = CookieEntry {
                        value,
                        domain: domain.map(str::to_string),
                        path: Some("/".to_string()),
                        expires,
                        host_only: false,
                        secure,
                        same_site: None,
                    };
                    changed |= self.insert(name, entry);
                }
                _ => {}
            }
        }
        if let Some(jct) = self.cookies.get("bili_jct")
            && !jct.is_expired()
            && self.csrf != jct.value
        {
            self.csrf = jct.value.clone();
            changed = true;
        }
        if changed {
            self.dirty = true;
            self.flush_if_stale();
        }
    }

    fn insert(&mut self, name: String, entry: CookieEntry) -> bool {
        let unchanged = self.cookies.get(&name).is_some_and(|existing| {
            existing.value == entry.value
                && existing.domain == entry.domain
                && existing.path == entry.path
                && existing.expires == entry.expires
                && existing.host_only == entry.host_only
                && existing.secure == entry.secure
                && existing.same_site == entry.same_site
        });
        if unchanged {
            return false;
        }
        self.cookies.insert(name, entry);
        true
    }

    /// Remove a cookie only when the server's Domain/Path match the stored one.
    fn remove_matching(&mut self, name: &str, domain: Option<&str>, path: &str) -> bool {
        let Some(entry) = self.cookies.get(name) else {
            return false;
        };
        let domain_matches = match (&entry.domain, domain) {
            // Legacy/global cookies have no domain; a deletion by name is safe.
            (None, _) => true,
            (Some(stored), Some(target)) => stored.eq_ignore_ascii_case(target),
            (Some(_), None) => false,
        };
        let path_matches = entry.path.as_deref().unwrap_or("/") == path;
        if domain_matches && path_matches {
            self.cookies.remove(name);
            true
        } else {
            false
        }
    }

    /// Extract Set-Cookie from response headers and update internal state.
    pub fn update_from_response(
        &mut self,
        url: &reqwest::Url,
        headers: &reqwest::header::HeaderMap,
    ) {
        let default_domain = url.host_str().map(str::to_ascii_lowercase);
        let default_path = default_cookie_path_for(url.path());
        let mut changed = false;

        for set_cookie in &headers.get_all("set-cookie") {
            let Ok(value) = set_cookie.to_str() else {
                continue;
            };
            let mut parts = value.split(';');
            let Some(cookie_part) = parts.next() else {
                continue;
            };
            let Some((name, raw_value)) = cookie_part.split_once('=') else {
                continue;
            };
            let name = name.trim().to_string();
            let value = raw_value.trim().to_string();

            let mut domain = None;
            let mut path = default_path.clone();
            let mut max_age = None;
            let mut expires = None;
            let mut secure = false;
            let mut same_site = None;
            for attribute in parts {
                let attribute = attribute.trim();
                if attribute.eq_ignore_ascii_case("secure") {
                    secure = true;
                    continue;
                }
                let Some((key, raw)) = attribute.split_once('=') else {
                    continue; // HttpOnly / other flags
                };
                match key.trim().to_ascii_lowercase().as_str() {
                    "domain" => {
                        domain = Some(raw.trim().trim_start_matches('.').to_ascii_lowercase());
                    }
                    "path" => path = raw.trim().to_string(),
                    "max-age" => max_age = raw.trim().parse::<i64>().ok(),
                    "expires" => expires = parse_http_date(raw.trim()),
                    "samesite" => same_site = parse_same_site(raw.trim()),
                    _ => {}
                }
            }

            let expires = max_age
                .map(|seconds| {
                    if seconds <= 0 {
                        0
                    } else {
                        now_unix().saturating_add(seconds)
                    }
                })
                .or(expires);
            let deletion_domain = domain.clone().or_else(|| default_domain.clone());
            let should_delete = value.is_empty() || expires.is_some_and(|e| e <= now_unix());
            if should_delete {
                // Delete only the cookie that matches the Domain/Path the
                // server is addressing; never wipe a same-name cookie from
                // another host or path.
                changed |= self.remove_matching(&name, deletion_domain.as_deref(), &path);
                continue;
            }

            let host_only = domain.is_none();
            let entry = CookieEntry {
                value,
                domain: domain.or(default_domain.clone()),
                path: Some(path),
                expires,
                host_only,
                secure,
                same_site,
            };
            changed |= self.insert(name, entry);
        }

        if let Some(jct) = self.cookies.get("bili_jct")
            && !jct.is_expired()
            && self.csrf != jct.value
        {
            self.csrf = jct.value.clone();
            changed = true;
        }
        if changed {
            self.dirty = true;
            self.flush_if_stale();
        }
    }

    /// Force flush cookies to disk.
    ///
    /// Writes to a temporary file and renames it into place so a crash cannot
    /// leave a half-written cookie jar. On Unix the file is created with mode
    /// `0600` from the start.
    pub fn flush(&mut self) {
        self.cookies.retain(|_, cookie| !cookie.is_expired());
        if persist_to_disk(&self.path, &self.cookies).is_ok() {
            self.dirty = false;
            self.last_save = Instant::now();
        }
    }

    fn flush_if_stale(&mut self) {
        if self.dirty && self.last_save.elapsed() >= SAVE_INTERVAL {
            self.flush();
        }
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}

fn persist_to_disk(
    path: &std::path::Path,
    cookies: &HashMap<String, CookieEntry>,
) -> std::io::Result<()> {
    #[derive(serde::Serialize)]
    struct PersistedRef<'a> {
        cookies: HashMap<&'a str, PersistedCookie>,
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let cookies = cookies
        .iter()
        .map(|(name, cookie)| (name.as_str(), PersistedCookie::from_entry(cookie)))
        .collect();
    let json =
        serde_json::to_string_pretty(&PersistedRef { cookies }).map_err(std::io::Error::other)?;

    // Write to a sibling temp file and rename: a crash/kill leaves either the
    // old jar or the new one, never a truncated JSON file.
    let file_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("cookies.json");
    let tmp = path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    write_private(&tmp, json.as_bytes())?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// Write `data` with owner-only permissions from the moment the file exists.
fn write_private(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        // `mode` only applies on creation; make sure a stale temp file from a
        // previous crash is also tightened.
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        file.write_all(data)?;
        file.sync_all()
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, data)
    }
}

/// Default cookie path (RFC 6265 5.1.4): the request path's directory.
fn default_cookie_path_for(request_path: &str) -> String {
    if !request_path.starts_with('/') {
        return "/".to_string();
    }
    match request_path.rfind('/') {
        Some(0) | None => "/".to_string(),
        Some(index) => request_path[..index].to_string(),
    }
}

fn parse_same_site(raw: &str) -> Option<SameSite> {
    match raw.to_ascii_lowercase().as_str() {
        "strict" => Some(SameSite::Strict),
        "lax" => Some(SameSite::Lax),
        "none" => Some(SameSite::None),
        _ => None,
    }
}

/// Parse an RFC 1123 HTTP date (`Sun, 06 Nov 1994 08:49:37 GMT`).
fn parse_http_date(raw: &str) -> Option<i64> {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let day: i64 = parts.get(1)?.trim_end_matches(',').parse().ok()?;
    let month = match parts.get(2)?.to_ascii_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => return None,
    };
    let year: i64 = parts.get(3)?.parse().ok()?;
    let mut time = parts.get(4)?.split(':');
    let hour: i64 = time.next()?.parse().ok()?;
    let minute: i64 = time.next()?.parse().ok()?;
    let second: i64 = time.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = (month + 9) % 12;
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

impl Drop for CookieStore {
    fn drop(&mut self) {
        // Only touch the disk if something actually changed: constructing a
        // client for a read-only command must not rewrite the cookie jar.
        if self.dirty {
            self.flush();
        }
    }
}

/// Shared cookie store type.
pub type SharedCookieStore = Arc<Mutex<CookieStore>>;

/// Create a new shared cookie store.
#[must_use]
pub fn new_shared(path: PathBuf) -> SharedCookieStore {
    Arc::new(Mutex::new(CookieStore::new(path)))
}

/// Default cookie path: ~/.config/bili/cookies.json
pub fn default_cookie_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    home.join(".config").join("bili").join("cookies.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cookie_path() -> PathBuf {
        let dir = std::env::temp_dir().join("bili_cookie_test");
        let _ = std::fs::create_dir_all(&dir);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        dir.join(format!("test_{ts}.json"))
    }

    fn header_for(store: &CookieStore, url: &str) -> Option<String> {
        store
            .build_cookie_header_for(&reqwest::Url::parse(url).unwrap())
            .map(|header| header.to_str().unwrap().to_string())
    }

    #[test]
    fn new_store_creates_empty() {
        let path = temp_cookie_path();
        let store = CookieStore::new(path.clone());
        assert!(!store.is_logged_in());
        assert_eq!(store.csrf_token(), "");
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn new_store_loads_legacy_format() {
        let path = temp_cookie_path();
        std::fs::write(
            &path,
            r#"{"cookies":{"SESSDATA":"test_token","bili_jct":"csrf123"}}"#,
        )
        .unwrap();

        let store = CookieStore::new(path.clone());
        assert!(store.is_logged_in());
        assert_eq!(store.csrf_token(), "csrf123");
        assert_eq!(store.get_sessdata(), Some("test_token"));
        assert!(header_for(&store, "https://api.bilibili.com/x").is_some());
        assert_eq!(header_for(&store, "https://example.com/x"), None);
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unchanged_store_does_not_rewrite_the_file() {
        let path = temp_cookie_path();
        std::fs::write(&path, "sentinel").unwrap();
        let store = CookieStore::new(path.clone());
        drop(store);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "sentinel");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn flush_persists_cookies() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        store.set_cookie("key", "value".to_string());
        store.flush();

        let content = std::fs::read_to_string(&path).unwrap();
        let loaded: Persisted = serde_json::from_str(&content).unwrap();
        let entry = loaded.cookies.get("key").unwrap();
        let value = match entry {
            PersistedCookie::Legacy(value) | PersistedCookie::Full { value, .. } => value,
        };
        assert_eq!(value, "value");
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn flush_writes_atomically_with_owner_only_permissions() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        store.set_from_cookie_string("SESSDATA=token; bili_jct=csrf");
        store.flush();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }

        // No temporary file for *this* cookie jar is left behind (other tests
        // in the same directory may have their own in-flight temp files).
        let dir = path.parent().unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy();
        let tmp_prefix = format!(".{file_name}.");
        let has_tmp = std::fs::read_dir(dir).unwrap().any(|entry| {
            let name = entry.unwrap().file_name().to_string_lossy().to_string();
            let is_tmp = std::path::Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("tmp"));
            name.starts_with(&tmp_prefix) && is_tmp
        });
        assert!(!has_tmp, "temporary cookie file was not renamed");

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_cookie_header_contains_cookies() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        store.set_from_cookie_string("SESSDATA=abc; bili_jct=xyz");
        let header = store.build_cookie_header().unwrap();
        let val = header.to_str().unwrap();
        assert!(val.contains("SESSDATA=abc"));
        assert!(val.contains("bili_jct=xyz"));
        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn build_cookie_header_empty_returns_none() {
        let path = temp_cookie_path();
        let store = CookieStore::new(path.clone());
        assert!(store.build_cookie_header().is_none());
        drop(store);
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
        drop(store);

        // Mutations are persisted on drop even if the 10s throttle has not elapsed.
        let loaded: Persisted =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(loaded.cookies.contains_key("SESSDATA"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cookies_are_scoped_by_domain_and_path() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());

        let url = reqwest::Url::parse("https://api.bilibili.com/x/player").unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=domain-cookie; Domain=.bilibili.com; Path=/x"),
        );
        headers.append(
            "set-cookie",
            HeaderValue::from_static("buvid3=host-cookie; Path=/"),
        );
        store.update_from_response(&url, &headers);

        assert_eq!(
            header_for(&store, "https://api.bilibili.com/x/player").as_deref(),
            Some("SESSDATA=domain-cookie; buvid3=host-cookie")
        );
        assert_eq!(
            header_for(&store, "https://www.bilibili.com/x/player").as_deref(),
            Some("SESSDATA=domain-cookie")
        );
        assert_eq!(
            header_for(&store, "https://api.bilibili.com/other").as_deref(),
            Some("buvid3=host-cookie")
        );
        assert_eq!(
            header_for(&store, "https://www.bilibili.com/xyz").as_deref(),
            None
        );
        assert_eq!(
            header_for(&store, "https://example.com/x/player").as_deref(),
            None
        );

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn expired_cookies_are_ignored_and_pruned() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        let url = reqwest::Url::parse("https://api.bilibili.com/x").unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=gone; Max-Age=0; Path=/"),
        );
        store.update_from_response(&url, &headers);
        assert!(!store.is_logged_in());

        headers.clear();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=old; Expires=Sun, 06 Nov 1994 08:49:37 GMT; Path=/"),
        );
        store.update_from_response(&url, &headers);
        assert!(!store.is_logged_in());

        store.flush();
        let loaded: Persisted =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!loaded.cookies.contains_key("SESSDATA"));

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn secure_cookies_require_https_and_same_site_is_kept() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        let url = reqwest::Url::parse("https://api.bilibili.com/x").unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "set-cookie",
            HeaderValue::from_static(
                "SESSDATA=secret; Domain=.bilibili.com; Path=/; Secure; SameSite=Lax",
            ),
        );
        store.update_from_response(&url, &headers);

        assert_eq!(header_for(&store, "http://api.bilibili.com/x"), None);
        assert_eq!(
            header_for(&store, "https://api.bilibili.com/x").as_deref(),
            Some("SESSDATA=secret")
        );
        let entry = store.cookies.get("SESSDATA").unwrap();
        assert!(entry.secure);
        assert_eq!(entry.same_site, Some(SameSite::Lax));

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn deletion_matches_domain_and_path() {
        let path = temp_cookie_path();
        let mut store = CookieStore::new(path.clone());
        let url = reqwest::Url::parse("https://api.bilibili.com/x").unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=keep; Domain=.bilibili.com; Path=/x"),
        );
        store.update_from_response(&url, &headers);

        headers.clear();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=; Domain=.bilibili.com; Path=/"),
        );
        store.update_from_response(&url, &headers);
        assert_eq!(
            store.get_sessdata(),
            Some("keep"),
            "wrong path must not delete"
        );

        headers.clear();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=; Domain=example.com; Path=/x"),
        );
        store.update_from_response(&url, &headers);
        assert_eq!(
            store.get_sessdata(),
            Some("keep"),
            "wrong domain must not delete"
        );

        headers.clear();
        headers.append(
            "set-cookie",
            HeaderValue::from_static("SESSDATA=; Domain=.bilibili.com; Path=/x"),
        );
        store.update_from_response(&url, &headers);
        assert_eq!(store.get_sessdata(), None);

        drop(store);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn default_cookie_path_uses_request_directory() {
        assert_eq!(default_cookie_path_for("/"), "/");
        assert_eq!(default_cookie_path_for("/x"), "/");
        assert_eq!(default_cookie_path_for("/x/player"), "/x");
        assert_eq!(default_cookie_path_for("/x/player/wbi"), "/x/player");
    }

    #[test]
    fn parse_http_date_matches_known_timestamp() {
        assert_eq!(
            parse_http_date("Sun, 06 Nov 1994 08:49:37 GMT"),
            Some(784_111_777)
        );
        assert_eq!(parse_http_date("not a date"), None);
    }

    #[test]
    fn days_from_civil_epoch() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
    }
}
