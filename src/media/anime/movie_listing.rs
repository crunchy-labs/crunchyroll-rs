use crate::categories::Category;
use crate::common::Request;
use crate::crunchyroll::Executor;
use crate::media::util::request_media;
use crate::media::{Media, PosterImages};
use crate::{Crunchyroll, Locale, Movie, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListingVersion {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    #[serde(rename = "guid")]
    pub id: String,

    pub audio_locale: Locale,

    pub original: bool,

    #[cfg(feature = "__test_strict")]
    variant: crate::StrictValue,
}

impl MovieListingVersion {
    /// Requests an actual [`MovieListing`] from this version.
    pub async fn movie_listing(&self) -> Result<MovieListing> {
        MovieListing::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            &self.id,
        )
        .await
    }
}

/// Metadata for a movie listing.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListing {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,
    pub extended_description: String,

    /// Sometimes none, sometimes not
    pub content_provider: Option<String>,

    pub movie_release_year: u32,

    /// May be [`None`] if requested by some functions like [`Crunchyroll::browse`]. You might have
    /// to re-request it to get the audio locale. Crunchyroll :)
    pub audio_locale: Option<Locale>,
    /// Sometimes empty, sometimes not. Not recommended to rely on this.
    pub subtitle_locales: Vec<Locale>,

    pub is_subbed: bool,
    pub is_dubbed: bool,

    pub images: PosterImages,

    /// Descriptors about the movie listing content, e.g. 'Violence' or 'Sexualized Imagery'.
    #[serde(default)]
    pub content_descriptors: Vec<String>,

    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub season_tags: Vec<String>,

    pub is_premium_only: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    /// All versions of this movie listing (same movie listing but each entry has a different language).
    pub versions: Vec<MovieListingVersion>,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    identifier: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    duration_ms: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    first_movie_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    hd_flag: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    playback: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    _type: Option<crate::StrictValue>,
}

impl MovieListing {
    /// Returns all movies for this movie listing.
    pub async fn movies(&self) -> Result<Vec<Movie>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}/movies",
            self.id
        );
        request_media(self.executor.clone(), endpoint).await
    }
}

#[async_trait::async_trait]
impl Media for MovieListing {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        Ok(request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0))
    }

    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
        for version in &mut self.versions {
            version.__set_executor(self.executor.clone()).await
        }
    }
}
