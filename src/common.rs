use serde::Deserialize;
use serde::de::DeserializeOwned;
use crate::Crunchyroll;
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

/// Every struct which implements this must provide a usable [`Crunchyroll`] instance.
pub trait Crunchy<'a>: DeserializeOwned {
    /// Returns a usable [`Crunchyroll`] instance.
    fn get_crunchyroll(&self) -> &'a Crunchyroll;
}

/// Check if further actions with the struct which implements this are available.
pub trait Available<'a>: Crunchy<'a> {
    /// Returns if the current episode, series, ... is available.
    fn available(&self) -> bool;
}

/// Every instance of the struct which this implements can be constructed by an id
#[async_trait::async_trait]
pub trait FromId<'a> {
    /// Creates a new [`Self`] by the provided id or returns an [`CrunchyrollError`] if something
    /// caused an issue.
    async fn from_id(crunchy: &'a Crunchyroll, id: String) -> Result<Self> where Self: Sized;
}
