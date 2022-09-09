use crate::common::{Image, Request};
use crate::error::Result;
use crate::{BulkResult, Crunchyroll, Locale};
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryImages {
    pub background: Vec<Image>,
    pub low: Vec<Image>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryLocalization {
    pub title: String,
    pub description: String,
    pub locale: Locale,
}

#[derive(Clone, Debug, Deserialize, Default, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Category {
    #[serde(rename = "tenant_category")]
    pub name: String,
    pub slug: String,

    pub images: CategoryImages,

    // there is a `sub_categories` in this struct when request with `include_subcategories=true`
    // but since this has no obvious use case, it's excluded here
    /// A human readable title & description about the category.
    pub localization: CategoryLocalization,
}

impl Crunchyroll {
    /// Returns all video categories.
    pub async fn categories(&self) -> Result<BulkResult<Category>> {
        let endpoint = "https://beta.crunchyroll.com/content/v1/tenant_categories";
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&[("locale", &self.executor.details.locale)]);
        self.executor.request(builder).await
    }
}
