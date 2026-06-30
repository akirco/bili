pub mod client;
pub mod credentials;
pub mod error;
pub mod wbi;

#[cfg(feature = "video")]
pub mod video;

#[cfg(feature = "user")]
pub mod user;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "comment")]
pub mod comment;

#[cfg(feature = "fav")]
pub mod fav;

#[cfg(feature = "danmaku")]
pub mod danmaku;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "history")]
pub mod history;

#[cfg(feature = "login")]
pub mod login;

#[cfg(feature = "subtitle")]
pub mod subtitle;

#[cfg(feature = "action")]
pub mod action;

pub use client::{BiliClient, FormBuilder, Params};
pub use credentials::Credentials;
pub use error::BiliError;