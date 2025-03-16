//! Media categories.

use crate::Result;
use crate::common::{Image, V2BulkResult};
use crate::crunchyroll::Executor;
use crate::{Crunchyroll, Locale, Request, enum_values};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
        // the following categories are sub-categories, they're not listed when calling
        // `Crunchyroll::categories`
        Harem = "harem"
        Historical = "historical"
        Idols = "idols"
        Isekai = "isekai"
        Mecha = "mecha"
        Mystery = "mystery"
        PostApocalyptic = "post-apocalyptic"
    }
}

impl Category {
    pub fn sub_categories() -> Vec<Category> {
        vec![
            Category::Harem,
            Category::Historical,
            Category::Idols,
            Category::Isekai,
            Category::Mecha,
            Category::Mystery,
            Category::PostApocalyptic,
        ]
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

/// An anime category / genre.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CategoryInformation {
    #[serde(skip)]
    executor: Arc<Executor>,

    #[serde(rename = "id")]
    pub category: Category,
    pub slug: String,

    pub images: CategoryInformationImages,

    /// A human readable title & description about the category.
    pub localization: CategoryInformationLocalization,
}

impl CategoryInformation {
    /// Get all sub-categories of this category.
    pub async fn sub_categories(&self) -> Result<Vec<SubCategoryInformation>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/discover/categories/{}/sub_categories",
            self.category
        );
        Ok(self
            .executor
            .get(endpoint)
            .apply_locale_query()
            .request::<V2BulkResult<SubCategoryInformation>>()
            .await?
            .data)
    }
}

/// A sub category of an anime category / genre.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SubCategoryInformation {
    #[serde(rename = "id")]
    pub category: Category,
    #[serde(rename = "parent_category_id")]
    pub parent_category: Category,

    pub slug: String,

    /// A human readable title & description about the category.
    pub localization: CategoryInformationLocalization,
}

impl Crunchyroll {
    /// Returns all video categories. Note that not all categories declared in [`Category`] are
    /// returned since some of them are sub-categories. Call [`Category::sub_categories`] to get a
    /// list of categories which are sub-categories.
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
