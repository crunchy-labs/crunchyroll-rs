extern crate core;

mod crunchyroll;
mod error;
mod internal;
mod media_collection;
mod common;

#[cfg(feature = "__test_strict")]
use internal::strict::StrictValue;

pub use crunchyroll::Crunchyroll;
pub use crunchyroll::Locale;

pub use common::FromId;

pub use media_collection::MovieListing;
pub use media_collection::Series;
