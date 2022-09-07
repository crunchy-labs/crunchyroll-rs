mod account;
mod auth;
mod crunchyroll;
mod error;
mod internal;
mod media_collection;
mod common;
mod media;
mod stream;
mod macros;

pub mod search;

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;

use auth::Executor;

pub use account::{
    Account,
    Wallpaper
};

pub use auth::{
    CrunchyrollBuilder,
    SessionToken
};

pub use crunchyroll::{
    Crunchyroll,
    Locale,
    MaturityRating
};

pub use common::{
    Collection,
    Panel,
    FromId,
    Playback,
    Streams
};

pub use media_collection::MovieListing;
pub use media_collection::Series;

pub use media::Episode;
pub use media::Movie;

pub use stream::{
    VideoStream,
    VideoVariants,
    VideoVariant,
    PlaybackStream,
    PlaybackVariants,
    PlaybackVariant
};
#[cfg(feature = "streaming")]
pub use stream::{
    DefaultStreams,
    VariantData,
    VariantSegment
};
