use crate::error::CrunchyrollError;
use crate::{Crunchyroll, EmptyJsonProxy, Executor, MediaCollection, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

/// A [`Crunchylist`] entry.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(panel))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrunchylistEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    pub list_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    /// Should only be [`MediaCollection::Series`] or [`MediaCollection::MovieListing`].
    pub panel: MediaCollection,
}

impl CrunchylistEntry {
    /// Delete this entry from the parent crunchylist.
    pub async fn delete(self, entry: &CrunchylistEntry) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}/{}/{}",
            self.executor.details.account_id.clone()?,
            entry.list_id,
            self.id
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request()
            .await?;
        Ok(())
    }
}

/// Representation of Crunchylists / custom lists you can create to store series or movies in.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Crunchylists {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub items: Vec<CrunchylistPreview>,

    pub total_public: u32,
    pub total_private: u32,
    pub max_private: u32,
}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct CrunchylistCreate {
    list_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    modified_at: DateTime<Utc>,

    total: u32,
}

impl Crunchylists {
    /// Create a new crunchylist. If a error is thrown which says that the maximum of private list
    /// is reached, check how many you currently have ([`Crunchylists::total_private`]) and how many
    /// are allowed ([`Crunchylists::max_private`]; usually 10).
    pub async fn create<S: AsRef<str>>(&self, title: S) -> Result<CrunchylistPreview> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}",
            self.executor.details.account_id.clone()?
        );
        let create_result: CrunchylistCreate = self
            .executor
            .post(endpoint)
            .json(&json!({ "title": title.as_ref() }))
            .apply_locale_query()
            .request()
            .await?;
        Ok(CrunchylistPreview {
            executor: self.executor.clone(),
            list_id: create_result.list_id,
            title: title.as_ref().to_string(),
            modified_at: create_result.modified_at,
            is_public: false,
            total: create_result.total,
        })
    }
}

/// A Crunchylist.
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[request(executor(items))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Crunchylist {
    #[serde(skip)]
    executor: Arc<Executor>,

    #[serde(skip)]
    pub id: String,

    pub items: Vec<CrunchylistEntry>,

    pub title: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    pub is_public: bool,
    pub total: u32,
    pub max: u32,
}

impl Crunchylist {
    /// Add a new entry to the current crunchylist. Note that [`MediaCollection::Season`] is not
    /// supported to add and will return an error.
    pub async fn add(&self, media: MediaCollection) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id.clone()?,
            self.id
        );
        let id = match media {
            MediaCollection::Series(series) => series.id,
            MediaCollection::Season(_) => {
                return Err(CrunchyrollError::Input("seasons are not supported".into()))
            }
            MediaCollection::Episode(episode) => episode.metadata.series_id,
            MediaCollection::MovieListing(movie_listing) => movie_listing.id,
            MediaCollection::Movie(movie) => movie.metadata.movie_listing_id,
        };
        self.executor
            .post(endpoint)
            .json(&json!({ "content_id": id }))
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }

    /// Rename the current crunchylist.
    pub async fn rename<S: AsRef<str>>(&self, name: S) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id.clone()?,
            self.id
        );
        self.executor
            .patch(endpoint)
            .json(&json!({ "title": name.as_ref() }))
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }

    /// Delete the current crunchylist.
    pub async fn delete(self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id.clone()?,
            self.id
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }
}

/// Abstraction of [`Crunchylist`]. Use [`CrunchylistPreview::crunchylist`] to get the "real"
/// [`Crunchylist`].
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrunchylistPreview {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub list_id: String,

    pub title: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    pub is_public: bool,
    pub total: u32,
}

impl CrunchylistPreview {
    /// Return the "real" [`Crunchylist`].
    pub async fn crunchylist(&self) -> Result<Crunchylist> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id.clone()?,
            self.list_id
        );
        let mut crunchylist: Crunchylist = self
            .executor
            .get(endpoint)
            .apply_locale_query()
            .request()
            .await?;
        crunchylist.id = self.list_id.clone();
        Ok(crunchylist)
    }
}

impl Crunchyroll {
    /// Return your crunchylists.
    pub async fn crunchylists(&self) -> Result<Crunchylists> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v1/custom-lists/{}",
            self.executor.details.account_id.clone()?
        );
        self.executor
            .get(endpoint)
            .apply_locale_query()
            .request()
            .await
    }
}
