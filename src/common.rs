use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use crate::{Crunchyroll, PlaybackStream, VideoStream};
use crate::crunchyroll::Executor;
use crate::error::Result;

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Deserialize)]
pub struct BulkResult<T> {
    pub items: Vec<T>,
    pub total: u32
}

/// The standard representation of images how the api returns them.
#[derive(Debug, Deserialize)]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
pub struct Image {
    pub source: String,
    #[serde(rename(deserialize = "type"))]
    pub image_type: String,
    pub height: u32,
    pub width: u32
}

/// Helper trait for [`Crunchyroll::request`] generic returns.
/// Must be implemented for every struct which is used as generic parameter for [`Crunchyroll::request`].
pub(crate) trait Request: DeserializeOwned {
    /// Set a usable [`Executor`] instance to the struct if required
    fn set_executor(&mut self, _: Arc<Executor>) {}

    /// When using the `__test_strict` feature, all fields starting and ending with `__` are removed
    /// from the json before converting it into a struct. These fields are usually not required. But
    /// if they're needed to be accessed, return the names of these fields with this method and they
    /// won't get touched.
    #[cfg(feature = "__test_strict")]
    fn not_clean_fields() -> Vec<String> {
        vec![]
    }
}

impl Request for () {}

/// Check if further actions with the struct which implements this are available.
pub trait Available {
    /// Returns if the current episode, series, ... is available.
    fn available(&self) -> bool;
}

/// Every instance of the struct which this implements can be constructed by an id
#[async_trait::async_trait]
pub trait FromId {
    /// Creates a new [`Self`] by the provided id or returns an [`CrunchyrollError`] if something
    /// caused an issue.
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self> where Self: Sized;
}

#[async_trait::async_trait]
pub trait Playback {
    async fn playback(&self) -> Result<PlaybackStream>;
}

#[async_trait::async_trait]
pub trait Streams {
    async fn streams(&self) -> Result<VideoStream>;
}
