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
    size(u32, "page_size") = Some(100),
    page(u32, "page") = None
}

impl Crunchyroll {
    /// Get the history which episodes / movies you've watched.
    pub async fn watch_history(
        &self,
        options: WatchHistoryOptions,
    ) -> Result<Vec<WatchHistoryEntry>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/watch-history/{}",
            self.executor.details.account_id.clone()?
        );
        Ok(self
            .executor
            .get(endpoint)
            .query(&options.into_query())
            .apply_locale_query()
            .request::<BulkWatchHistoryResult>()
            .await?
            .items)
    }

    /// Clear your watch history.
    pub async fn clear_watch_history(&self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/watch-history/{}",
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
