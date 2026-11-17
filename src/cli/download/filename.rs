//! Turning a video title into a safe file name.

/// Keep room for `.partN` / `.video.m4s` suffixes and stay below the common
/// 255-**byte** filename limit (Chinese titles use 3 bytes per character).
pub const MAX_FILENAME_BYTES: usize = 180;

/// Replace characters that are illegal in file names so a video title can be
/// used as a path component safely.
///
/// Control characters become `_`, trailing dots and spaces are trimmed, and
/// the result is truncated on a character boundary to
/// [`MAX_FILENAME_BYTES`]. An empty result falls back to `video`.
#[must_use]
pub fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = sanitized.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        return "video".to_string();
    }
    let truncated: String = trimmed
        .char_indices()
        .take_while(|(index, c)| index + c.len_utf8() <= MAX_FILENAME_BYTES)
        .map(|(_, c)| c)
        .collect();
    let truncated = truncated.trim().trim_end_matches('.').trim();
    if truncated.is_empty() {
        "video".to_string()
    } else {
        truncated.to_string()
    }
}
