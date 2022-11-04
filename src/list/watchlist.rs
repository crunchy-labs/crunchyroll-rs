use crate::error::CrunchyrollError;
use crate::{
    enum_values, options, Crunchyroll, EmptyJsonProxy, Executor, MediaCollection, Request, Result,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(panel))]
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
            _ => Err(CrunchyrollError::Internal(
                "panel is not series nor movie listing".into(),
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
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

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct BulkWatchlistResult {
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
            "https://www.crunchyroll.com/content/v1/{}/watchlist",
            self.executor.details.account_id
        );
        Ok(self
            .executor
            .get(endpoint)
            .query(&options.into_query())
            .query(&[language_field])
            .apply_locale_query()
            .request::<BulkWatchlistResult>()
            .await?
            .items)
    }
}

macro_rules! add_to_watchlist {
    ($(#[doc = $add:literal] #[doc = $as:literal] $s:path);*) => {
        $(
            impl $s {
                #[doc = $add]
                pub async fn add_to_watchlist(&self) -> Result<()> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/watchlist/{}", self.executor.details.account_id);
                    let _: EmptyJsonProxy = self.executor.post(endpoint)
                        .json(&json!({"content_id": &self.id}))
                        .query(&[("locale", &self.executor.details.locale)])
                        .request()
                        .await?;
                    Ok(())
                }

                #[doc = $as]
                pub async fn into_watchlist_entry(&self) -> Result<Option<SimpleWatchlistEntry>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/watchlist/{}/{}", self.executor.details.account_id, self.id);
                    let builder = isahc::Request::get(endpoint).body(()).unwrap();
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
    crate::Media<crate::media::Series>;
    #[doc = "Add this movie to your watchlist."]
    #[doc = "Check and convert this movie to a watchlist entry (to check if this movie was watched before)."]
    crate::Media<crate::media::MovieListing>
}

async fn mark_favorite_watchlist(
    executor: &Arc<Executor>,
    id: String,
    favorite: bool,
) -> Result<()> {
    let endpoint = format!(
        "https://www.crunchyroll.com/content/v1/watchlist/{}/{}",
        executor.details.account_id, id
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
        "https://www.crunchyroll.com/content/v1/watchlist/{}/{}",
        executor.details.account_id, id
    );
    executor
        .delete(endpoint)
        .json(&json!({}))
        .apply_locale_query()
        .request::<EmptyJsonProxy>()
        .await?;
    Ok(())
}
