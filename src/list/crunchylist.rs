use crate::error::CrunchyrollError;
use crate::{Crunchyroll, EmptyJsonProxy, Executor, MediaCollection, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

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

    pub panel: MediaCollection,
}

impl CrunchylistEntry {
    pub async fn delete(&self, entry: &CrunchylistEntry) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}/{}",
            self.executor.details.account_id, entry.list_id, self.id
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request()
            .await?;
        Ok(())
    }
}

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
    pub async fn create(&self, title: String) -> Result<CrunchylistPreview> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}",
            self.executor.details.account_id
        );
        let create_result: CrunchylistCreate = self
            .executor
            .post(endpoint)
            .json(&json!({ "title": &title }))
            .apply_locale_query()
            .request()
            .await?;
        Ok(CrunchylistPreview {
            executor: self.executor.clone(),
            list_id: create_result.list_id,
            title,
            modified_at: create_result.modified_at,
            is_public: false,
            total: create_result.total,
        })
    }
}

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
    pub async fn add(&self, media: MediaCollection) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
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

    pub async fn rename(&self, name: String) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
        );
        self.executor
            .patch(endpoint)
            .json(&json!({ "title": name }))
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }

    pub async fn delete(self) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
        );
        self.executor
            .delete(endpoint)
            .apply_locale_query()
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }
}

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
    pub async fn crunchylist(&self) -> Result<Crunchylist> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.list_id
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
    pub async fn crunchylists(&self) -> Result<Crunchylists> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}",
            self.executor.details.account_id
        );
        self.executor
            .get(endpoint)
            .apply_locale_query()
            .request()
            .await
    }
}
