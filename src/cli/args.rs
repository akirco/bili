//! Helpers for parsing CLI-style argument values that clap cannot express.

/// A comma-separated id list such as `1, 2,3` failed to parse.
#[derive(Debug, thiserror::Error)]
#[error("invalid fid '{value}': {source}")]
pub struct InvalidFid {
    /// The offending element, trimmed.
    pub value: String,
    /// The underlying integer parse error.
    pub source: std::num::ParseIntError,
}

/// Parse a comma-separated list of integers, tolerating spaces around commas.
///
/// # Errors
///
/// Returns [`InvalidFid`] naming the first element that is not an integer.
pub fn parse_fids(raw: &str) -> Result<Vec<i64>, InvalidFid> {
    raw.split(',')
        .map(|part| {
            let trimmed = part.trim();
            trimmed.parse::<i64>().map_err(|source| InvalidFid {
                value: trimmed.to_string(),
                source,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_spaces_around_commas() {
        assert_eq!(parse_fids("1, 2,3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn rejects_non_numeric_elements() {
        let error = parse_fids("1,x").unwrap_err();
        assert_eq!(error.value, "x");
    }
}
