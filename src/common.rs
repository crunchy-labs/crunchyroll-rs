use crate::Executor;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) use crunchyroll_rs_internal::Request;

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct BulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
    pub total: u32,
}

/// Just like [`BulkResult`] but without [`BulkResult::total`] because some request does not have
/// this field (but should?!).
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrappyBulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
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
#[async_trait::async_trait]
pub trait Request: Send {
    /// Set a usable [`Executor`] instance to the struct if required
    async fn __set_executor(&mut self, _: Arc<Executor>) {}
}

/// Implement [`Request`] for cases where only the request must be done without needing an
/// explicit result.
impl Request for () {}

impl<K: Send, V: Send> Request for HashMap<K, V> {}

impl Request for serde_json::Value {}
