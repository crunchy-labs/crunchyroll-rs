use crate::common::FromId;
use crate::media::{Panel, VideoCollection};
use crate::{Crunchyroll, EmptyJsonProxy, Executor, Request, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request, FromId)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CrunchylistEntry {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    pub list_id: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    pub panel: Panel,
}

impl CrunchylistEntry {
    pub async fn delete(&self, entry: &CrunchylistEntry) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}/{}",
            self.executor.details.account_id, entry.list_id, self.id
        );
        let builder = self
            .executor
            .client
            .delete(endpoint)
            .query(&[("locale", &self.executor.details.locale)]);
        self.executor.request::<EmptyJsonProxy>(builder).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Crunchylists {
    #[serde(skip)]
    executor: Arc<Executor>,

    #[default(Vec::new())]
    pub items: Vec<CrunchylistPreview>,

    pub total_public: u32,
    pub total_private: u32,
    pub max_private: u32,
}

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
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
        let builder = self
            .executor
            .client
            .post(endpoint)
            .json(&json!({ "title": &title }))
            .query(&[("locale", &self.executor.details.locale)]);
        let create_result: CrunchylistCreate = self.executor.request(builder).await?;
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

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Crunchylist {
    #[serde(skip)]
    executor: Arc<Executor>,

    #[serde(skip)]
    pub id: String,

    #[default(Vec::new())]
    pub items: Vec<CrunchylistEntry>,

    pub title: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified_at: DateTime<Utc>,

    pub is_public: bool,
    pub total: u32,
    pub max: u32,
}

impl Crunchylist {
    pub async fn add(&self, collection: impl VideoCollection) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
        );
        let builder = self
            .executor
            .client
            .post(endpoint)
            .json(&json!({"content_id": collection.id()}))
            .query(&[("locale", &self.executor.details.locale)]);
        self.executor.request::<EmptyJsonProxy>(builder).await?;
        Ok(())
    }

    pub async fn rename(&self, name: String) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
        );
        let builder = self
            .executor
            .client
            .patch(endpoint)
            .query(&[("locale", &self.executor.details.locale)])
            .json(&json!({ "title": name }));
        self.executor.request::<EmptyJsonProxy>(builder).await?;
        Ok(())
    }

    pub async fn delete(self) -> Result<()> {
        let endpoint = format!(
            "https://beta.crunchyroll.com/content/v1/custom-lists/{}/{}",
            self.executor.details.account_id, self.id
        );
        let builder = self
            .executor
            .client
            .delete(endpoint)
            .query(&[("content_id", &self.id)]);
        self.executor.request::<EmptyJsonProxy>(builder).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
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
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &self.executor.details.locale)]);
        let mut crunchylist: Crunchylist = self.executor.request(builder).await?;
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
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &self.executor.details.locale)]);
        self.executor.request(builder).await
    }
}
