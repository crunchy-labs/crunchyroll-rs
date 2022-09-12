use crate::common::CrappyBulkResult;
use crate::media::Panel;
use crate::{options, Crunchyroll, EmptyJsonProxy, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct WatchHistoryEntry {
    pub id: String,
    pub parent_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub date_played: DateTime<Utc>,
    pub playhead: u32,
    pub fullywatched: bool,

    pub panel: Panel,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct BulkWatchHistoryResult {
    #[default(Vec::new())]
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
    pub async fn watch_history(
        &self,
        options: WatchHistoryOptions,
    ) -> Result<CrappyBulkResult<WatchHistoryEntry>> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/watch-history/{}",
            self.executor.details.account_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[(
                "locale".to_string(),
                self.executor.details.locale.to_string(),
            )]));
        let bulk_watch_history_result: BulkWatchHistoryResult =
            self.executor.request(builder).await?;
        Ok(CrappyBulkResult {
            items: bulk_watch_history_result.items,
        })
    }

    pub async fn clear_watch_history(&self) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/watch-history/{}",
            self.executor.details.account_id
        );
        let builder = self.executor.client.delete(endpoint).query(&[(
            "locale".to_string(),
            self.executor.details.locale.to_string(),
        )]);
        self.executor.request::<EmptyJsonProxy>(builder).await?;
        Ok(())
    }
}
