use crate::common::V2BulkResult;
use crate::error::Error;
use crate::{
    enum_values, options, Crunchyroll, EmptyJsonProxy, Executor, MediaCollection, Request, Result,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// A item in your watchlist.
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[request(executor(panel))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct WatchlistEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub new: bool,

    pub is_favorite: bool,

    pub never_watched: bool,
    pub fully_watched: bool,

    pub playhead: u32,

    /// Should only be [`MediaCollection::Series`] or [`MediaCollection::MovieListing`].
    pub panel: MediaCollection,
}

impl WatchlistEntry {
    /// Mark this entry as favorite on your watchlist. The argument this function takes, says if the
    /// entry should be marked (`true`) or unmarked (`false`) as favorite.
    pub async fn mark_favorite(&mut self, favorite: bool) -> Result<()> {
        mark_favorite_watchlist(&self.executor, self.get_id()?, favorite).await?;
        self.is_favorite = favorite;

        Ok(())
    }

    /// Remove this entry from your watchlist.
    pub async fn remove(self) -> Result<()> {
        let id = self.get_id()?;
        remove_from_watchlist(self.executor, id).await
    }

    /// Get the media id of the series / movie listing which represents this entry.
    fn get_id(&self) -> Result<String> {
        match self.panel.clone() {
            MediaCollection::Series(series) => Ok(series.id),
            MediaCollection::MovieListing(movie_listing) => Ok(movie_listing.id),
            _ => Err(Error::Internal {
                message: "panel is not series nor movie listing".to_string(),
            }),
        }
    }
}

/// A simplified version of [`WatchlistEntry`].
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
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
    /// Mark this entry as favorite on your watchlist. The argument this function takes, says if the
    /// entry should be marked (`true`) or unmarked (`false`) as favorite.
    pub async fn mark_favorite(&mut self, favorite: bool) -> Result<()> {
        mark_favorite_watchlist(&self.executor, self.id.clone(), favorite).await?;
        self.is_favorite = favorite;

        Ok(())
    }

    /// Remove this entry from your watchlist.
    pub async fn remove(self) -> Result<()> {
        remove_from_watchlist(self.executor, self.id).await
    }
}

enum_values! {
    /// Filter how to sort watchlist entries when querying.
    pub enum WatchlistSort {
        Updated = "date_updated"
        Watched = "date_watched"
        Added = "date_added"
        Alphabetical = "alphabetical"
    }
}

enum_values! {
    /// Order how to sort watchlist entries when querying.
    pub enum WatchlistOrder {
        Newest = "desc"
        Oldest = "asc"
    }
}

enum_values! {
    /// If queried watchlist entries should be subbed or dubbed.
    pub enum WatchlistLanguage {
        Subbed = "subbed"
        Dubbed = "dubbed"
    }
}

options! {
    /// Options how to query the watchlist.
    WatchlistOptions;
    order(WatchlistOrder, "order") = Some(WatchlistOrder::Newest),
    sort(WatchlistSort, "sort_by") = None,
    media_type(crate::media::MediaType, "type") = None,
    language(WatchlistLanguage, "language") = None,
    only_favorites(bool, "only_favorites") = Some(false)
}

impl Crunchyroll {
    /// Returns your watchlist.
    pub async fn watchlist(&self, mut options: WatchlistOptions) -> Result<Vec<WatchlistEntry>> {
        let true_string = true.to_string();
        let language_field = if let Some(language) = options.language {
            match language {
                WatchlistLanguage::Subbed => ("is_subbed", true_string.as_str()),
                WatchlistLanguage::Dubbed => ("is_dubbed", true_string.as_str()),
                _ => ("", ""),
            }
        } else {
            ("", "")
        };
        options.language = None;

        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/discover/{}/watchlist",
            self.executor.details.account_id.clone()?
        );
        Ok(self
            .executor
            .get(endpoint)
            .query(&options.into_query())
            .query(&[language_field])
            .apply_locale_query()
            .request::<V2BulkResult<WatchlistEntry>>()
            .await?
            .data)
    }
}

macro_rules! add_to_watchlist {
    ($(#[doc = $add:literal] #[doc = $as:literal] $s:path);*) => {
        $(
            impl $s {
                #[doc = $add]
                pub async fn add_to_watchlist(&self) -> Result<()> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/{}/watchlist", self.executor.details.account_id.clone()?);
                    let _: EmptyJsonProxy = self.executor.post(endpoint)
                        .json(&json!({"content_id": &self.id}))
                        .apply_locale_query()
                        .request()
                        .await?;
                    Ok(())
                }

                #[doc = $as]
                pub async fn into_watchlist_entry(&self) -> Result<Option<SimpleWatchlistEntry>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/{}/watchlist", self.executor.details.account_id.clone()?);
                    Ok(self.executor
                        .get(endpoint)
                        .query(&[("content_ids", &self.id)])
                        .apply_locale_query()
                        .request::<V2BulkResult<SimpleWatchlistEntry>>()
                        .await?
                        .data
                        .get(0)
                        .cloned())
                }
            }
        )*
    }
}

add_to_watchlist! {
    #[doc = "Add this series to your watchlist."]
    #[doc = "Check and convert this series to a watchlist entry (to check if this series was watched before)."]
    crate::media::Series;
    #[doc = "Add this movie to your watchlist."]
    #[doc = "Check and convert this movie to a watchlist entry (to check if this movie was watched before)."]
    crate::media::MovieListing
}

async fn mark_favorite_watchlist(
    executor: &Arc<Executor>,
    id: String,
    favorite: bool,
) -> Result<()> {
    let endpoint = format!(
        "https://www.crunchyroll.com/content/v2/{}/watchlist/{}",
        executor.details.account_id.clone()?,
        id
    );
    executor
        .patch(endpoint)
        .json(&json!({ "is_favorite": favorite }))
        .request::<EmptyJsonProxy>()
        .await?;
    Ok(())
}

async fn remove_from_watchlist(executor: Arc<Executor>, id: String) -> Result<()> {
    let endpoint = format!(
        "https://www.crunchyroll.com/content/v2/{}/watchlist/{}",
        executor.details.account_id.clone()?,
        id
    );
    executor
        .delete(endpoint)
        .json(&json!({}))
        .apply_locale_query()
        .request::<EmptyJsonProxy>()
        .await?;
    Ok(())
}
