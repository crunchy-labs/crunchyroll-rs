use crate::crunchyroll::Executor;
use crate::media::util::request_media;
use crate::media::{AdBreak, Media, ThumbnailImages};
use crate::{Crunchyroll, MovieListing, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Metadata for a movie.
#[allow(dead_code, non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Movie {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,

    #[serde(alias = "listing_id")]
    pub movie_listing_id: String,
    pub movie_listing_title: String,
    #[serde(default)]
    pub movie_listing_slug_title: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[serde(serialize_with = "crate::internal::serde::serialize_duration_to_millis")]
    #[default(Duration::try_milliseconds(0).unwrap())]
    pub duration: Duration,

    pub images: ThumbnailImages,

    /// Descriptors about the movie content, e.g. 'Violence' or 'Sexualized Imagery'.
    #[serde(default)]
    pub content_descriptors: Vec<String>,

    #[default(DateTime::< Utc >::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::< Utc >::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,
    /// Is "available" or "not available"
    pub availability_status: String,

    /// **Only [`Some`] if the account is non-premium**. Contains ad breaks.
    #[serde(default)]
    pub ad_breaks: Option<Vec<AdBreak>>,

    #[cfg(feature = "__test_strict")]
    streams_link: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    #[serde(alias = "media_type")]
    type_: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    audio_locale: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    availability_starts: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    availability_ends: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    SeriesGUID: Option<crate::StrictValue>,
}

impl Movie {
    /// Returns the parent movie listing of this movie.
    pub async fn movie_listing(&self) -> Result<MovieListing> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/movie_listings/{}",
            self.movie_listing_id
        );
        Ok(request_media(self.executor.clone(), endpoint)
            .await?
            .remove(0))
    }
}

impl Media for Movie {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        Ok(request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/movies/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0))
    }

    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }
}
