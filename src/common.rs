use crate::{Executor, Result};
use futures_util::{Stream, StreamExt};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

pub(crate) use crunchyroll_rs_internal::Request;

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(data))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct V2BulkResult<T, M = serde_json::Map<String, serde_json::Value>>
where
    T: Default + DeserializeOwned + Request,
    M: Default + DeserializeOwned + Send,
{
    pub data: Vec<T>,
    #[serde(default)]
    pub total: u32,

    #[serde(default)]
    pub(crate) meta: M,
}

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct V2TypeBulkResult<T: Default + DeserializeOwned + Request> {
    #[serde(rename = "type")]
    pub(crate) result_type: String,
    #[serde(alias = "count")]
    pub(crate) total: u32,
    pub(crate) items: Vec<T>,
}

#[derive(Clone)]
pub(crate) struct PaginationOptions {
    pub(crate) executor: Arc<Executor>,
    pub(crate) start: u32,
    pub(crate) page: u32,
    pub(crate) page_size: u32,
    pub(crate) query: Vec<(String, String)>,
    pub(crate) extra: BTreeMap<&'static str, String>,
}

#[allow(clippy::type_complexity)]
pub struct Pagination<T: Default + DeserializeOwned + Request> {
    data: Vec<T>,

    init: bool,
    next_fn: Box<
        dyn FnMut(
            PaginationOptions,
        ) -> Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>,
    >,
    next_state: Option<Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>>,

    paginator_options: PaginationOptions,

    count: u32,
    total: u32,
}

impl<T: Default + DeserializeOwned + Request> Stream for Pagination<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if this.count < this.total || !this.init {
            if !this.data.is_empty() {
                this.count += 1;
                return Poll::Ready(Some(Ok(this.data.remove(0))));
            }

            if this.next_state.is_none() {
                let f = this.next_fn.as_mut();
                let options = &mut this.paginator_options;
                options.start = this.count;
                options.page += 1;
                this.next_state = Some(f(options.clone()));
            }

            let fut = this.next_state.as_mut().unwrap();
            match Pin::new(fut).poll(cx) {
                Poll::Ready(result) => {
                    this.next_state = None;
                    match result {
                        Ok((t, total)) => {
                            this.data = t;
                            this.total = total;

                            if !this.init {
                                this.init = true;
                            }

                            Pin::new(this).poll_next(cx)
                        }
                        Err(e) => Poll::Ready(Some(Err(e))),
                    }
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            Poll::Ready(None)
        }
    }
}

impl<T: Default + DeserializeOwned + Request> Unpin for Pagination<T> {}

impl<T: Default + DeserializeOwned + Request> Pagination<T> {
    pub(crate) fn new<F>(
        pagination_fn: F,
        executor: Arc<Executor>,
        query: Option<Vec<(String, String)>>,
        extra: Option<Vec<(&'static str, String)>>,
    ) -> Self
    where
        F: FnMut(
                PaginationOptions,
            )
                -> Pin<Box<dyn Future<Output = Result<(Vec<T>, u32)>> + Send + 'static>>
            + Send
            + 'static,
    {
        Self {
            data: vec![],
            init: false,
            next_fn: Box::new(pagination_fn),
            next_state: None,
            paginator_options: PaginationOptions {
                executor,
                start: 0,
                page: 0,
                page_size: 20,
                query: query.unwrap_or_default(),
                extra: extra.map_or(BTreeMap::new(), BTreeMap::from_iter),
            },
            count: 0,
            total: 0,
        }
    }

    /// Set the amount of pages fetched when needed. Only recommended to change if you want a big
    /// batch of data (> 100). Make sure that the size is never 0 as this will cause a dead loop.
    pub fn page_size(&mut self, size: u32) {
        self.paginator_options.page_size = size
    }

    /// Return the total amount of items which can be fetched.
    pub async fn total(&mut self) -> u32 {
        if !self.init {
            StreamExt::next(self).await;
        }
        self.total
    }
}

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct BulkResult<T: Default + DeserializeOwned + Request> {
    pub items: Vec<T>,
    pub total: u32,
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
impl<K: Send, V: Send> Request for serde_json::Map<K, V> {}
impl Request for serde_json::Value {}
