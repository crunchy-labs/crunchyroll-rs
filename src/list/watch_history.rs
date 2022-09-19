use crate::common::CrappyBulkResult;
use crate::{options, Crunchyroll, EmptyJsonProxy, MediaCollection, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Entry of your watchlist.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct WatchHistoryEntry {
    pub id: String,
    pub parent_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub date_played: DateTime<Utc>,
    pub playhead: u32,
    pub fullywatched: bool,

    /// Should always be [`MediaCollection::Episode`] or [`MediaCollection::Movie`].
    pub panel: MediaCollection,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct BulkWatchHistoryResult {
    items: Vec<WatchHistoryEntry>,

    #[cfg(feature = "__test_strict")]
    #[serde(default)]
    #[default(crate::StrictValue::default())]
    // field does not appear when `items` is `[]` (empty)
    next_page: crate::StrictValue,
}

options! {
    WatchHistoryOptions;
    size(u32, "size") = None,
    page(u32, "page") = Some(100)
}

impl Crunchyroll {
    /// Get the history which episodes / movies you've watched.
    pub async fn watch_history(
        &self,
        options: WatchHistoryOptions,
    ) -> Result<CrappyBulkResult<WatchHistoryEntry>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/watch-history/{}",
            self.executor.details.account_id
        );
        let bulk_watch_history_result: BulkWatchHistoryResult = self
            .executor
            .get(endpoint)
            .query(&options.to_query())
            .apply_locale_query()
            .request()
            .await?;
        Ok(CrappyBulkResult {
            items: bulk_watch_history_result.items,
        })
    }

    /// Clear your watch history.
    pub async fn clear_watch_history(&self) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/watch-history/{}",
            self.executor.details.account_id
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }
}
