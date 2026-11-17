//! Bilibili API client library and CLI.
//!
//! The library is organised by API domain, one module per area
//! ([`video`], [`user`], [`search`], …), each gated behind a Cargo feature of
//! the same name so a consumer only compiles what it calls. [`BiliClient`] is
//! the single entry point: it owns the HTTP client, the cookie jar and the WBI
//! key cache, and hands out the per-domain clients via `client.video()`,
//! `client.user()`, and so on.
//!
//! Response models live in the same module as their client: `video::VideoInfo`
//! sits next to `video::VideoClient`.
//!
//! The example below only touches the always-available core; the per-domain
//! clients appear with their feature:
//!
//! ```no_run
//! use bili::prelude::*;
//!
//! # fn main() -> Result<(), BiliError> {
//! let client = BiliClient::builder().build()?;
//! // With the `video` feature: `client.video().info(None, Some(bvid)).await?`
//! println!("logged in: {}", client.is_logged_in());
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod cookie;
pub mod error;
mod serde_util;
pub mod wbi;

#[cfg(feature = "action")]
pub mod action;
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "comment")]
pub mod comment;
#[cfg(feature = "danmaku")]
pub mod danmaku;
#[cfg(feature = "fav")]
pub mod fav;
#[cfg(feature = "history")]
pub mod history;
#[cfg(feature = "live")]
pub mod live;
#[cfg(feature = "login")]
pub mod login;
#[cfg(feature = "search")]
pub mod search;
#[cfg(feature = "subtitle")]
pub mod subtitle;
#[cfg(feature = "user")]
pub mod user;
#[cfg(feature = "video")]
pub mod video;

pub use client::{BiliClient, BiliClientBuilder, BiliResponse, FormBuilder, FormData, Params};
pub use cookie::CookieStore;
pub use error::BiliError;

/// Common imports for convenient glob importing.
///
/// # Usage
///
/// ```
/// use bili::prelude::*;
/// ```
///
/// # Contents
///
/// The core client types plus one client per enabled API domain. Response
/// models are *not* included — import those from their domain module
/// (`bili::video::VideoInfo`), which keeps names like `Video` out of the
/// caller's namespace.
pub mod prelude {
    pub use crate::{BiliClient, BiliClientBuilder, BiliError, BiliResponse, CookieStore};

    #[cfg(feature = "action")]
    pub use crate::action::ActionClient;
    #[cfg(feature = "audio")]
    pub use crate::audio::AudioClient;
    #[cfg(feature = "comment")]
    pub use crate::comment::CommentClient;
    #[cfg(feature = "danmaku")]
    pub use crate::danmaku::DanmakuClient;
    #[cfg(feature = "fav")]
    pub use crate::fav::FavClient;
    #[cfg(feature = "history")]
    pub use crate::history::HistoryClient;
    #[cfg(feature = "live")]
    pub use crate::live::LiveClient;
    #[cfg(feature = "login")]
    pub use crate::login::LoginClient;
    #[cfg(feature = "search")]
    pub use crate::search::SearchClient;
    #[cfg(feature = "subtitle")]
    pub use crate::subtitle::SubtitleClient;
    #[cfg(feature = "user")]
    pub use crate::user::UserClient;
    #[cfg(feature = "video")]
    pub use crate::video::VideoClient;
}
