use std::sync::Arc;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use crate::Crunchyroll;
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
    fn executor_control(&mut self) -> Option<&mut dyn ExecutorControl> {
        None
    }
}

impl Request for () {}

/// Every struct which implements this must provide a usable [`Executor`] instance.
pub(crate) trait ExecutorControl {
    /// Returns a usable [`Executor`] instance.
    fn get_executor(&self) -> Arc<Executor>;

    fn set_executor(&mut self, executor: Arc<Executor>);
}

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

/*#[async_trait::async_trait]
pub trait Playback {
    async fn playback(&self) -> Result<Stream>;
}

 */