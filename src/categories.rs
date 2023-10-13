//! Media categories.

use crate::common::{Image, V2BulkResult};
use crate::Result;
use crate::{enum_values, Crunchyroll, Locale, Request};
use serde::{Deserialize, Serialize};

enum_values! {
    /// Video categories / genres.
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

impl From<CategoryInformation> for Category {
    fn from(category_information: CategoryInformation) -> Self {
        category_information.category
    }
}

/// Images for [`CategoryInformation`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryInformationImages {
    pub background: Vec<Image>,
    pub low: Vec<Image>,
}

/// Human readable text about a category.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryInformationLocalization {
    pub title: String,
    pub description: String,
    pub locale: Locale,
}

/// A anime category / genre.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryInformation {
    #[serde(rename = "id")]
    pub category: Category,
    pub slug: String,

    pub images: CategoryInformationImages,

    /// A human readable title & description about the category.
    pub localization: CategoryInformationLocalization,
}

impl Crunchyroll {
    /// Returns all video categories.
    pub async fn categories(&self) -> Result<Vec<CategoryInformation>> {
        let endpoint = "https://www.crunchyroll.com/content/v2/discover/categories";
        Ok(self
            .executor
            .get(endpoint)
            .apply_locale_query()
            .request::<V2BulkResult<CategoryInformation>>()
            .await?
            .data)
    }
}
