pub mod action;
pub mod audio;
pub mod comment;
pub mod danmaku;
pub mod fav;
pub mod history;
pub mod login;
pub mod search;
pub mod subtitle;
pub mod user;
pub mod video;

pub use action::*;
pub use audio::*;
pub use comment::*;
pub use danmaku::*;
pub use fav::*;
pub use history::*;
pub use login::*;
pub use search::*;
pub use subtitle::*;
pub use user::*;
pub use video::*;

/// Treat JSON `null` like a missing field when a `Vec`/map field is declared
/// as non-optional (B 站在空列表场景下偶尔返回 `null`）。
pub(crate) fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}
