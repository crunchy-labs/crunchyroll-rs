extern crate core;

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

use crunchyroll::Executor;

pub use crunchyroll::Crunchyroll;
pub use crunchyroll::Locale;

pub use common::{
    Collection,
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
