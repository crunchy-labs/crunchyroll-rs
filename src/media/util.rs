use crate::common::{Request, V2BulkResult};
use crate::crunchyroll::Executor;
use crate::Result;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub(crate) async fn request_media<T: Default + DeserializeOwned + Request>(
    executor: Arc<Executor>,
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
