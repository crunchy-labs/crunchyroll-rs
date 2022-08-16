extern crate core;

mod crunchyroll;
mod error;
mod internal;
mod media_collection;
mod common;
mod media;
mod stream;

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;

use crunchyroll::Executor;

pub use crunchyroll::Crunchyroll;
pub use crunchyroll::Locale;

pub use common::{
    FromId,
    Playback,
    Streams
};

pub use media_collection::MovieListing;
pub use media_collection::Series;

pub use media::Episode;
pub use media::Movie;

pub use stream::{
    Stream,
    VideoVariants,
    PlaybackVariants,
    VariantData,
    VariantSegment
};
