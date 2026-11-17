//! Small serde helpers shared by several response-model modules.

/// Treat JSON `null` like a missing field when a `Vec`/map field is declared
/// as non-optional (B 站在空列表场景下偶尔返回 `null`）。
///
/// Usage:
///
/// ```ignore
/// #[serde(default, deserialize_with = "crate::serde_util::null_as_default")]
/// pub tags: Vec<String>,
/// ```
// Only `comment`, `fav`, `live` and `user` have fields that need this; gate on
// the same set so a lean build does not warn about an unused helper.
#[cfg(any(
    feature = "comment",
    feature = "fav",
    feature = "live",
    feature = "user"
))]
pub(crate) fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
