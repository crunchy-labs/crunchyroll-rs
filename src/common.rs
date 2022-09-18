use crate::Executor;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) use proc_macro::Request;

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Debug, Deserialize, smart_default::SmartDefault)]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct BulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
    pub total: u32,
}

impl<T: Default + DeserializeOwned + Request> Request for BulkResult<T> {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        for item in self.items.iter_mut() {
            item.__set_executor(executor.clone())
        }
    }
}

/// Just like [`BulkResult`] but without [`BulkResult::total`] because some request does not have
/// this field (but should?!).
#[derive(Debug, Deserialize, smart_default::SmartDefault)]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrappyBulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
}

impl<T: Default + DeserializeOwned + Request> Request for CrappyBulkResult<T> {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        for item in self.items.iter_mut() {
            item.__set_executor(executor.clone())
        }
    }
}

/// The standard representation of images how the api returns them.
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Image {
    pub source: String,
    #[serde(rename(deserialize = "type"))]
    pub image_type: String,
    pub height: u32,
    pub width: u32,
}

/// Helper trait for [`Crunchyroll::request`] generic returns.
/// Must be implemented for every struct which is used as generic parameter for [`Crunchyroll::request`].
#[doc(hidden)]
pub trait Request {
    /// Set a usable [`Executor`] instance to the struct if required
    fn __set_executor(&mut self, _: Arc<Executor>) {}
}

/// Implement [`Request`] for cases where only the request must be done without needing an
/// explicit result.
impl Request for () {}

impl<K, V> Request for HashMap<K, V> {}

impl Request for serde_json::Value {}
