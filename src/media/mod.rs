mod episode;
mod media_collection;
mod media_impl;
mod movie;
mod movie_listing;
mod season;
mod series;
mod stream;
#[cfg(any(feature = "hls-stream", feature = "dash-stream"))]
mod streaming;
mod util;

pub use episode::*;
pub use media_collection::*;
pub use media_impl::*;
pub use movie::*;
pub use movie_listing::*;
pub use season::*;
pub use series::*;
pub use stream::*;
#[cfg(any(feature = "hls-stream", feature = "dash-stream"))]
pub use streaming::*;

use crate::common::{Request, V2BulkResult};
use crate::crunchyroll::Executor;
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

pub(crate) async fn request_media<T: Default + serde::de::DeserializeOwned + Request>(
    executor: std::sync::Arc<Executor>,
    endpoint: String,
) -> Result<Vec<T>> {
    let result: V2BulkResult<T> = executor
        .get(endpoint)
        .apply_locale_query()
        .apply_preferred_audio_locale_query()
        .request()
        .await?;
    Ok(result.data)
}
