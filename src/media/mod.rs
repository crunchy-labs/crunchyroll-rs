//! All media items like series, episodes or movies.

mod anime;
mod r#impl;
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
use crate::internal::sealed::Sealed;
use crate::{Crunchyroll, Result};
use std::sync::Arc;

/// Trait every media struct ([`Series`], [`Season`], [`Episode`], [`MovieListing`], [`Movie`],
/// [`MusicVideo`], [`Concert`]) implements.
pub trait Media: Sealed + Into<MediaCollection> {
    fn from_id(
        crunchyroll: &Crunchyroll,
        id: impl AsRef<str> + Send,
    ) -> impl Future<Output = Result<Self>>
    where
        Self: Sized;

    #[doc(hidden)]
    fn __set_executor(&mut self, executor: Arc<Executor>) -> impl Future<Output = ()>;

    #[doc(hidden)]
    fn __apply_fixes(&mut self) -> impl Future<Output = ()> {
        async move {}
    }

    #[doc(hidden)]
    #[cfg(feature = "experimental-stabilizations")]
    fn __apply_experimental_stabilizations(&mut self) -> impl Future<Output = ()> {
        async move {}
    }
}

impl Crunchyroll {
    pub async fn media_from_id<M: Media>(&self, id: impl AsRef<str> + Send) -> Result<M> {
        M::from_id(self, id).await
    }

    pub async fn media_collection_from_id<S: AsRef<str>>(&self, id: S) -> Result<MediaCollection> {
        MediaCollection::from_id(self, id).await
    }
}
