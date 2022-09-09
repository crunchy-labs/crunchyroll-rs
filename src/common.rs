use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
use crate::{Crunchyroll, Executor};
use crate::{Locale, PlaybackStream, VideoStream};
use chrono::{DateTime, Duration, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) use macros::{Available, FromId, Playback, Request};

/// Contains a variable amount of items and the maximum / total of item which are available.
/// Mostly used when fetching pagination results.
#[derive(Debug, Deserialize)]
#[serde(bound = "T: Request + DeserializeOwned")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(
    not(feature = "__test_strict"),
    serde(default),
    derive(smart_default::SmartDefault)
)]
pub struct BulkResult<T: Request + DeserializeOwned> {
    #[cfg_attr(not(feature = "__test_strict"), default(Vec::new()))]
    pub items: Vec<T>,
    pub total: u32,
}

impl<T: Request + DeserializeOwned> Request for BulkResult<T> {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        for item in self.items.iter_mut() {
            item.__set_executor(executor.clone())
        }
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SearchMetadata {
    pub score: f64,
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
    #[cfg_attr(
        feature = "__test_strict",
        serde(serialize_with = "crate::internal::serde::serialize_duration_to_millis")
    )]
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
    #[cfg_attr(
        feature = "__test_strict",
        serde(serialize_with = "crate::internal::serde::serialize_duration_to_millis")
    )]
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

#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct CollectionImages {
    pub thumbnail: Option<Vec<Vec<Image>>>,
    pub poster_tall: Option<Vec<Vec<Image>>>,
    pub poster_wide: Option<Vec<Vec<Image>>>,
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

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub last_public: DateTime<Utc>,

    pub series_metadata: Option<SeriesMetadata>,
    pub movie_listing_metadata: Option<MovieListingMetadata>,
    pub episode_metadata: Option<EpisodeMetadata>,

    pub images: Option<PanelImages>,

    #[serde(alias = "type")]
    #[cfg(feature = "__test_strict")]
    collection_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: crate::StrictValue,
}

/// The standard representation of images how the api returns them.
#[derive(Clone, Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Image {
    pub source: String,
    #[serde(rename(deserialize = "type"))]
    pub image_type: String,
    pub height: u32,
    pub width: u32,
}

/// Helper trait for [`Crunchyroll::request`] generic returns.
/// Must be implemented for every struct which is used as generic parameter for [`Crunchyroll::request`].
#[doc(hidden)]
pub trait Request {
    /// Set a usable [`Executor`] instance to the struct if required
    fn __set_executor(&mut self, _: Arc<Executor>) {}

    /// Get the [`Executor`] instance of the struct which implements this trait (if available).
    fn __get_executor(&self) -> Option<Arc<Executor>> {
        None
    }
}

/// Implement [`Request`] for cases where only the request must be done without needing an
/// explicit result.
impl Request for () {}

impl<K, V> Request for HashMap<K, V> {}

/// Check if further actions with the struct which implements this are available.
pub trait Available: Request {
    /// Returns if the current episode, series, ... is available.
    fn available(&self) -> bool;
}

/// Every instance of the struct which implements this can be constructed by an id
#[async_trait::async_trait]
pub trait FromId {
    /// Creates a new [`Self`] by the provided id or returns an [`CrunchyrollError`] if something
    /// caused an issue.
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self>
    where
        Self: Sized;
}

/// Provides playback streams for episodes or movies. Playback streams are mostly used to provide
/// trailers for an episode / movie.
#[async_trait::async_trait]
pub trait Playback {
    /// Returns the playback streams.
    async fn playback(&self) -> Result<PlaybackStream>;
}

/// Provides video streams for episodes or movies. This streams are what the end user sees when
/// watching a video on Crunchyroll.
#[async_trait::async_trait]
pub trait Streams {
    /// Returns the streams.
    async fn streams(&self) -> Result<VideoStream>;
}
