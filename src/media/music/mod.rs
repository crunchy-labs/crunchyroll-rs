mod artist;
mod concert;
mod r#impl;
mod music_video;
mod util;

pub use artist::*;
pub use concert::*;
pub use music_video::*;

use crate::Request;
use serde::Deserialize;

/// A music genre.
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MusicGenre {
    pub id: String,

    pub display_value: String,
}
