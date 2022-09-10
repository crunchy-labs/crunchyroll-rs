use crate::common::{Image, Request};
use crate::error::Result;
use crate::{enum_values, BulkResult, Crunchyroll, Locale};
use serde::Deserialize;

enum_values! {
    #[doc = "Video categories / genres"]
    pub enum Category {
        Action = "action"
        Adventure = "adventure"
        Comedy = "comedy"
        Drama = "drama"
        Fantasy = "fantasy"
        Music = "music"
        Romance = "romance"
        SciFi = "sci-fi"
        Seinen = "seinen"
        Shojo = "shojo"
        Shonen = "shonen"
        SliceOfLife = "slice-of-life"
        Sports = "sports"
        Supernatural = "supernatural"
        Thriller = "thriller"
    }
}

impl From<TenantCategory> for Category {
    fn from(tenant_category: TenantCategory) -> Self {
        Self::from(tenant_category.name)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategoryImages {
    pub background: Vec<Image>,
    pub low: Vec<Image>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategoryLocalization {
    pub title: String,
    pub description: String,
    pub locale: Locale,
}

#[derive(Clone, Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct TenantCategory {
    #[serde(rename = "tenant_category")]
    pub name: String,
    pub slug: String,

    pub images: TenantCategoryImages,

    // there is a `sub_categories` in this struct when request with `include_subcategories=true`
    // but since this has no obvious use case, it's excluded here
    /// A human readable title & description about the category.
    pub localization: TenantCategoryLocalization,
}

impl Crunchyroll {
    /// Returns all video categories.
    pub async fn tenant_categories(&self) -> Result<BulkResult<TenantCategory>> {
        let endpoint = "https://beta.crunchyroll.com/content/v1/tenant_categories";
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &self.executor.details.locale)]);
        self.executor.request(builder).await
    }
}
