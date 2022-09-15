use crate::common::Image;
use crate::media::Playback;
use crate::{Executor, Locale, Request};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SearchMetadata {
    // `None` if queried by `Crunchyroll::by_query`
    pub last_public: Option<DateTime<Utc>>,
    // `None` if queried by `Crunchyroll::by_query`
    pub rank: Option<u32>,

    pub score: f64,
    // `None` if not queried by `Series::similar` or `MovieListing::similar`
    pub popularity_score: Option<f64>
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SeriesMetadata {
    pub extended_description: String,

    pub series_launch_year: Option<u32>,

    pub episode_count: u32,
    pub season_count: u32,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    pub audio_locales: Vec<Locale>,
    pub subtitle_locales: Vec<Locale>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    tenant_categories: Option<crate::StrictValue>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListingMetadata {
    // wtf is this again
    pub first_movie_id: String,

    pub extended_description: String,

    pub movie_release_year: u32,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub subtitle_locales: Vec<Locale>,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct EpisodeMetadata {
    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    pub season_number: u32,

    // usually the same as episode_number, just as string
    pub episode: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    pub episode_number: u32,
    // usually also the same as episode_number, I don't know the purpose of this
    pub sequence_number: u32,
    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub episode_air_date: DateTime<Utc>,
    // the same as episode_air_date as far as I can see
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub upload_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_starts: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_ends: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,
    // would be very useful, but is (currently) always empty
    pub audio_locale: String,
    pub subtitle_locales: Vec<Locale>,

    pub is_clip: bool,
    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub eligible_region: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    identifier: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    tenant_categories: Option<crate::StrictValue>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieMetadata {
    pub movie_listing_id: String,

    pub movie_listing_title: String,
    pub movie_listing_slug_title: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CollectionImages {
    pub thumbnail: Option<Vec<Vec<Image>>>,
    pub poster_tall: Option<Vec<Vec<Image>>>,
    pub poster_wide: Option<Vec<Vec<Image>>>,
    pub promo_image: Option<Vec<Vec<Image>>>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Default, Request, Playback)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Collection {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id_option")]
    pub stream_id: Option<String>,
    #[serde(rename = "playback")]
    pub playback_id: Option<String>,
    pub external_id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub promo_title: String,
    pub description: String,
    pub promo_description: String,

    pub new: bool,
    pub new_content: bool,

    pub search_metadata: SearchMetadata,

    pub series_metadata: Option<SeriesMetadata>,
    pub movie_listing_metadata: Option<MovieListingMetadata>,
    pub episode_metadata: Option<EpisodeMetadata>,
    pub movie_metadata: Option<MovieMetadata>,

    pub images: Option<CollectionImages>,

    #[serde(alias = "type")]
    #[cfg(feature = "__test_strict")]
    collection_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: crate::StrictValue,
}

type PanelImages = CollectionImages;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request, Playback)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Panel {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id_option")]
    pub stream_id: Option<String>,
    #[serde(rename = "playback")]
    pub playback_id: Option<String>,
    pub external_id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub promo_title: String,
    pub description: String,
    pub promo_description: String,

    pub series_metadata: Option<SeriesMetadata>,
    pub movie_listing_metadata: Option<MovieListingMetadata>,
    pub episode_metadata: Option<EpisodeMetadata>,
    pub movie_metadata: Option<MovieMetadata>,

    pub images: Option<PanelImages>,

    #[serde(alias = "type")]
    #[cfg(feature = "__test_strict")]
    collection_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new_content: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: crate::StrictValue,
}
