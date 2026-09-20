use super::executor::Executor;
use crate::Request;
#[cfg(feature = "__test_strict")]
use crate::error::{Error, ErrorKind};
use crate::error::{Result, check_request};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, RequestBuilder};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub(crate) struct ExecutorRequestBuilder {
    executor: Arc<Executor>,
    builder: RequestBuilder,
}

impl ExecutorRequestBuilder {
    pub(crate) fn new(executor: Arc<Executor>, builder: RequestBuilder) -> Self {
        Self { executor, builder }
    }

    pub(crate) fn query<T: Serialize + ?Sized>(mut self, query: &T) -> ExecutorRequestBuilder {
        self.builder = self.builder.query(query);

        self
    }

    pub(crate) fn apply_locale_query(self) -> ExecutorRequestBuilder {
        let locale = self.executor.details.locale.clone();
        self.query(&[("locale", locale)])
    }

    pub(crate) fn apply_preferred_audio_locale_query(self) -> ExecutorRequestBuilder {
        if let Some(locale) = self.executor.details.preferred_audio_locale.clone() {
            self.query(&[("preferred_audio_language", locale)])
        } else {
            self
        }
    }

    pub(crate) fn apply_ratings_query(self) -> ExecutorRequestBuilder {
        self.query(&[("ratings", "true")])
    }

    pub(crate) fn json<T: Serialize + ?Sized>(mut self, json: &T) -> ExecutorRequestBuilder {
        self.builder = self.builder.json(json);

        self
    }

    pub(crate) fn header<K, V>(mut self, key: K, value: V) -> ExecutorRequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.builder = self.builder.header(key, value);

        self
    }

    pub(crate) async fn request<T: Request + DeserializeOwned>(self) -> Result<T> {
        self.executor.request(self.builder).await
    }

    pub(crate) async fn request_static<T: Request + DeserializeOwned>(self) -> Result<Option<T>> {
        let raw_result = self.request_raw(false).await?;
        if raw_result
            .windows(8)
            .any(move |window| window == b"</Error>")
        {
            Ok(None)
        } else {
            Ok(serde_json::from_slice(raw_result.as_slice())?)
        }
    }

    pub(crate) async fn request_raw(mut self, auth: bool) -> Result<Vec<u8>> {
        if auth {
            self.builder = self.executor.auth_req(self.builder).await?;
        }

        #[cfg(feature = "middleware")]
        if let Some(middleware) = &self.executor.middleware {
            let req = self.builder.build()?;
            let url = req.url().to_string();

            return Ok(middleware
                .lock()
                .await
                .call(crate::middleware::MiddlewareContext::new(
                    &self.executor.client,
                    req,
                ))
                .await
                .map_err(|e| crate::internal::middleware::middleware_error_to_error(e, url))?
                .bytes()
                .await?
                .to_vec());
        }
        Ok(self.builder.send().await?.bytes().await?.to_vec())
    }
}

/// Make a request from the provided builder.
pub(super) async fn request<T: Request + DeserializeOwned>(
    client: &Client,
    req: RequestBuilder,
    #[cfg(feature = "middleware")] middleware: Option<
        &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
    >,
) -> Result<T> {
    let req = req.build()?;
    #[cfg(not(feature = "middleware"))]
    let resp = client.execute(req).await?;
    #[cfg(feature = "middleware")]
    let resp = {
        use std::ops::DerefMut;
        let url = req.url().to_string();
        if let Some(middleware) = middleware {
            middleware
                .lock()
                .await
                .deref_mut()
                .call(crate::middleware::MiddlewareContext::new(client, req))
                .await
                .map_err(|e| crate::internal::middleware::middleware_error_to_error(e, url))?
        } else {
            client.execute(req).await?
        }
    };

    #[cfg(not(feature = "__test_strict"))]
    {
        check_request(resp).await
    }
    #[cfg(feature = "__test_strict")]
    {
        let url = resp.url().to_string();
        let result = check_request(resp).await?;

        let cleaned = clean_request(result);
        // convert the map back to a string. by doing this, the error message contains the span
        // where the error occurred which improves debuggability
        let cleaned_string = serde_json::to_string(&cleaned)?;
        serde_json::from_str(&cleaned_string).map_err(|e| {
            Error::error_from_other_error_and_url(
                e,
                ErrorKind::Decode {
                    content: Some(cleaned_string.into_bytes()),
                },
                url,
            )
        })
    }
}

/// Removes all fields which are starting and ending with `__` from a map (which is usually the
/// response of a request). Some fields can be excluded from this process by providing the field
/// names in `not_clean_fields`.
#[cfg(feature = "__test_strict")]
fn clean_request(
    mut map: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    for (key, value) in map.clone() {
        if key.starts_with("__") && key.ends_with("__") {
            map.remove(key.as_str());
        } else if let Some(object) = value.as_object() {
            map.insert(
                key,
                serde_json::to_value(clean_request(object.clone())).unwrap(),
            );
        } else if let Some(array) = value.as_array() {
            map.insert(
                key,
                serde_json::to_value(clean_request_array(array.clone())).unwrap(),
            );
        }
    }
    map
}

#[cfg(feature = "__test_strict")]
fn clean_request_array(mut arr: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    for (i, item) in arr.clone().iter().enumerate() {
        if let Some(object) = item.as_object() {
            arr[i] = serde_json::to_value(clean_request(object.clone())).unwrap();
        } else if let Some(array) = item.as_array() {
            arr[i] = serde_json::to_value(clean_request_array(array.clone())).unwrap();
        }
    }
    arr
}
