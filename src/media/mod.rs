//! All media items like series, episodes or movies.

mod anime;
mod media_collection;
mod music;
mod shared;
mod stream;
mod util;

pub use anime::*;
pub use media_collection::*;
pub use music::*;
pub use shared::*;
pub use stream::*;

use crate::crunchyroll::Executor;
use crate::{Crunchyroll, Result};
use std::sync::Arc;

crate::enum_values! {
    /// Type of media.
    pub enum MediaType {
        Series = "series"
        Movie = "movie_listing"
    }
}

/// Trait every media struct ([`Series`], [`Season`], [`Episode`], [`MovieListing`], [`Movie`],
/// [`MusicVideo`], [`Concert`]) implements.
#[allow(async_fn_in_trait)]
pub trait Media {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self>
    where
        Self: Sized;

    #[doc(hidden)]
    async fn __set_executor(&mut self, executor: Arc<Executor>);

    #[doc(hidden)]
    async fn __apply_fixes(&mut self) {}

    #[doc(hidden)]
    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_experimental_stabilizations(&mut self) {}
}

impl Crunchyroll {
    pub async fn media_from_id<M: Media>(&self, id: impl AsRef<str> + Send) -> Result<M> {
        M::from_id(self, id).await
    }

    pub async fn media_collection_from_id<S: AsRef<str>>(&self, id: S) -> Result<MediaCollection> {
        MediaCollection::from_id(self, id).await
    }
}
