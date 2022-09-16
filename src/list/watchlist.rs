use crate::media::{MediaType, Panel};
use crate::{
    enum_values, options, BulkResult, Crunchyroll, EmptyJsonProxy, Executor, MovieListing, Request,
    Result, Series,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct WatchlistEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub new: bool,
    pub new_content: bool,

    pub is_favorite: bool,

    pub playhead: u32,
    pub never_watched: bool,
    pub completion_status: bool,

    pub panel: Panel,
}

impl Request for WatchlistEntry {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        self.panel.__set_executor(executor);
    }
}

impl WatchlistEntry {
    pub async fn mark_favorite(&mut self, favorite: bool) -> Result<()> {
        mark_favorite_watchlist(&self.executor, &self.panel.id, favorite).await?;
        self.is_favorite = favorite;

        Ok(())
    }

    pub async fn remove(self) -> Result<()> {
        remove_from_watchlist(self.executor, self.panel.id).await
    }
}

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SimpleWatchlistEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,

    pub is_favorite: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub date_added: DateTime<Utc>,
}

impl SimpleWatchlistEntry {
    pub async fn mark_favorite(&mut self, favorite: bool) -> Result<()> {
        mark_favorite_watchlist(&self.executor, &self.id, favorite).await?;
        self.is_favorite = favorite;

        Ok(())
    }

    pub async fn remove(self) -> Result<()> {
        remove_from_watchlist(self.executor, self.id).await
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct BulkWatchlistResult {
    #[default(Vec::new())]
    items: Vec<WatchlistEntry>,

    total: u32,

    #[cfg(feature = "__test_strict")]
    total_before_filter: u32,
}

enum_values! {
    pub enum WatchlistSort {
        Updated = "date_updated"
        Watched = "date_watched"
        Added = "date_added"
        Alphabetical = "alphabetical"
    }
}

enum_values! {
    pub enum WatchlistOrder {
        Newest = "desc"
        Oldest = "asc"
    }
}

enum_values! {
    pub enum WatchlistLanguage {
        Subbed = "subbed"
        Dubbed = "dubbed"
    }
}

options! {
    WatchlistOptions;
    order(WatchlistOrder, "order") = Some(WatchlistOrder::Newest),
    sort(WatchlistSort, "sort_by") = None,
    media_type(MediaType, "type") = None,
    language(WatchlistLanguage, "language") = None,
    only_favorits(bool, "only_favorites") = Some(false)
}

impl Crunchyroll {
    pub async fn watchlist(
        &self,
        mut options: WatchlistOptions,
    ) -> Result<BulkResult<WatchlistEntry>> {
        let language_field = if let Some(language) = options.language {
            match language {
                WatchlistLanguage::Subbed => ("is_subbed".to_string(), true.to_string()),
                WatchlistLanguage::Dubbed => ("is_dubbed".to_string(), true.to_string()),
                _ => ("".to_string(), "".to_string()),
            }
        } else {
            ("".to_string(), "".to_string())
        };
        options.language = None;

        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/{}/watchlist",
            self.executor.details.account_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&options.to_query(&[
                (
                    "locale".to_string(),
                    self.executor.details.locale.to_string(),
                ),
                language_field,
            ]));
        let bulk_watchlist_result: BulkWatchlistResult = self.executor.request(builder).await?;
        Ok(BulkResult {
            items: bulk_watchlist_result.items,
            total: bulk_watchlist_result.total,
        })
    }
}

macro_rules! add_to_watchlist {
    ($(#[doc = $add:literal] #[doc = $as:literal] $s:ident);*) => {
        $(
            impl $s {
                #[doc = $add]
                pub async fn add_to_watchlist(&self) -> Result<()> {
                    let endpoint = format!("https://beta.crunchyroll.com/content/v1/watchlist/{}", self.executor.details.account_id);
                    let builder = self.executor.client.post(endpoint)
                        .json(&json!({"content_id": &self.id}))
                        .query(&[("locale", &self.executor.details.locale)]);
                    self.executor.request::<EmptyJsonProxy>(builder).await?;
                    Ok(())
                }

                #[doc = $as]
                pub async fn into_watchlist_entry(&self) -> Result<Option<SimpleWatchlistEntry>> {
                    let endpoint = format!("https://beta.crunchyroll.com/content/v1/watchlist/{}/{}", self.executor.details.account_id, self.id);
                    let builder = self.executor.client.get(endpoint);
                    let result: serde_json::Value = self.executor.request(builder).await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        let mut entry: SimpleWatchlistEntry = serde_json::from_value(as_map.get(&self.id).unwrap().clone())?;
                        entry.executor = self.executor.clone();
                        Ok(Some(entry))
                    }
                }
            }
        )*
    }
}

add_to_watchlist! {
    #[doc = "Add this series to your watchlist."]
    #[doc = "Check and convert this series to a watchlist entry (to check if this series was watched before)."]
    Series;
    #[doc = "Add this movie to your watchlist."]
    #[doc = "Check and convert this movie to a watchlist entry (to check if this movie was watched before)."]
    MovieListing
}

async fn mark_favorite_watchlist(
    executor: &Arc<Executor>,
    id: &String,
    favorite: bool,
) -> Result<()> {
    let endpoint = format!(
        "https://beta.crunchyroll.com/content/v1/watchlist/{}/{}",
        executor.details.account_id, id
    );
    let builder = executor
        .client
        .patch(endpoint)
        .json(&json!({ "is_favorite": favorite }));
    executor.request::<EmptyJsonProxy>(builder).await?;
    Ok(())
}

async fn remove_from_watchlist(executor: Arc<Executor>, id: String) -> Result<()> {
    let endpoint = format!(
        "https://beta.crunchyroll.com/content/v1/watchlist/{}/{}",
        executor.details.account_id, id
    );
    let builder = executor
        .client
        .delete(endpoint)
        .json(&json!({}))
        .query(&[("locale", &executor.details.locale)]);
    executor.request::<EmptyJsonProxy>(builder).await?;
    Ok(())
}
