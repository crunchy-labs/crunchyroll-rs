mod anime;
mod media_collection;
mod music;
mod shared;
mod stream;
#[cfg(any(feature = "hls-stream", feature = "dash-stream"))]
mod streaming;
mod util;

pub use anime::*;
pub use media_collection::*;
pub use music::*;
pub use shared::*;
pub use stream::*;
#[cfg(any(feature = "hls-stream", feature = "dash-stream"))]
pub use streaming::*;

use crate::{Crunchyroll, Result};

crate::enum_values! {
    pub enum MediaType {
        Series = "series"
        Movie = "movie_listing"
    }
}

#[async_trait::async_trait]
pub trait Media {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self>
    where
        Self: Sized;

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
