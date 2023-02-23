use crate::categories::Category;
use crate::crunchyroll::Executor;
use crate::media::anime::util::real_dedup_vec;
use crate::media::util::request_media;
use crate::media::{Media, PosterImages};
use crate::{Crunchyroll, Locale, Result, Season};
use serde::Deserialize;
use std::sync::Arc;

/// Metadata for a series.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Series {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub channel_id: String,

    /// Sometimes none, sometimes not
    pub content_provider: Option<String>,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,
    pub extended_description: String,

    pub series_launch_year: Option<u32>,

    pub episode_count: u32,
    pub season_count: u32,
    #[serde(default)]
    pub media_count: u32,

    #[serde(default)]
    pub season_tags: Vec<String>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub audio_locales: Vec<Locale>,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub subtitle_locales: Vec<Locale>,

    pub images: PosterImages,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    #[serde(default)]
    pub keywords: Vec<String>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new_content: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    search_metadata: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    _type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
}

impl Series {
    /// Returns all series seasons.
    pub async fn seasons(&self) -> Result<Vec<Season>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}/seasons",
            self.id
        );
        request_media(self.executor.clone(), endpoint).await
    }
}

#[async_trait::async_trait]
impl Media for Series {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        Ok(request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/series/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0))
    }

    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_experimental_stabilizations(&mut self) {
        if self.executor.fixes.locale_name_parsing {
            if let Ok(seasons) = self.seasons().await {
                let mut locales = vec![];
                for mut season in seasons {
                    locales.extend(season.available_versions().await.unwrap_or_default());
                    locales.extend(season.audio_locales)
                }
                real_dedup_vec(&mut locales);

                self.audio_locales = locales
            }
        }
    }
}
