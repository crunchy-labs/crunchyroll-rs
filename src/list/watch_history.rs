use crate::common::{Pagination, V2BulkResult};
use crate::{Crunchyroll, EmptyJsonProxy, MediaCollection, Request, Result};
use chrono::{DateTime, Utc};
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};

/// Entry of your watchlist.
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
#[request(executor(panel))]
pub struct WatchHistoryEntry {
    /// Id of the episode or movie entry.
    pub id: String,
    pub parent_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub date_played: DateTime<Utc>,
    pub playhead: u32,
    pub fully_watched: bool,

    /// Should always be [`MediaCollection::Episode`] or [`MediaCollection::Movie`].
    pub panel: MediaCollection,
}

impl Crunchyroll {
    /// Get the history which episodes / movies you've watched.
    pub fn watch_history(&self) -> Pagination<WatchHistoryEntry> {
        Pagination::new(
            |options| {
                async move {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content/v2/{}/watch-history",
                        options.executor.details.account_id.clone()?
                    );
                    let result = options
                        .executor
                        .get(endpoint)
                        .query(&[("page", options.page), ("page_size", options.page_size)])
                        .apply_locale_query()
                        .request::<V2BulkResult<WatchHistoryEntry>>()
                        .await?;
                    Ok((result.data, result.total))
                }
                .boxed()
            },
            self.executor.clone(),
            None,
            None,
        )
    }

    /// Clear your watch history.
    pub async fn clear_watch_history(&self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/{}/watch-history",
            self.executor.details.account_id.clone()?
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }
}
