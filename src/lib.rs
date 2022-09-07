pub mod account;
pub mod auth;
pub mod common;
pub mod crunchyroll;
pub mod error;
pub mod media;
pub mod media_collection;
pub mod search;
pub mod stream;

// internal
mod internal;
mod macros;

use auth::Executor;
pub use common::{BulkResult, Collection, FromId, Playback, Streams};
pub use crunchyroll::{Crunchyroll, Locale};
pub use media::{Episode, Movie};
pub use media_collection::{MovieListing, Series};
#[cfg(feature = "streaming")]
pub use stream::{DefaultStreams, VariantData, VariantSegment};
pub use stream::{PlaybackStream, VideoStream};

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;
