use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use crate::common::{FromId, Image, Request};
use crate::media::{Playback, Streams, VideoStream};
use crate::{Executor, Locale, Result};

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct EpisodeImages {
    pub thumbnail: Vec<Vec<Image>>,
}

/// This struct represents a Crunchyroll episode.
#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Playback, FromId)]
#[from_id(multiple(crate::Season))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Episode {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id")]
    pub stream_id: String,
    #[serde(rename = "playback")]
    pub playback_id: String,
    pub channel_id: String,
    // whatever this is
    pub production_episode_id: String,
    // not really needed ig
    pub listing_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub seo_title: String,
    pub description: String,
    pub seo_description: String,

    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    pub season_number: u32,

    // usually the same as episode_number, just as string
    pub episode: String,
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

    #[serde(default)]
    // the api result simply does not contain this field if the episode is the last of its season.
    // classic crunchyroll moment
    pub next_episode_id: Option<String>,
    #[serde(default)]
    // the api result simply does not contain this field if the episode is the last of its season.
    // classic crunchyroll moment
    pub next_episode_title: Option<String>,

    pub season_tags: Vec<String>,

    pub images: EpisodeImages,

    pub hd_flag: bool,
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
    media_type: crate::StrictValue,
}

impl Request for Episode {
    fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor
    }

    fn __get_executor(&self) -> Option<Arc<Executor>> {
        Some(self.executor.clone())
    }
}

#[async_trait::async_trait]
impl Streams for Episode {
    async fn streams(&self) -> Result<VideoStream> {
        let endpoint = format!(
            "https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams",
            self.executor.details.bucket, self.stream_id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&self.executor.media_query());

        self.executor.request(builder).await
    }
}

type MovieImages = EpisodeImages;

/// This struct represents a Crunchyroll movie.
#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request, Playback, FromId)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Movie {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "playback")]
    pub playback_id: String,
    pub channel_id: String,
    // id of corresponding movie_listing object
    pub listing_id: String,

    pub slug: String,
    pub title: String,
    pub movie_listing_title: String,
    pub slug_title: String,
    pub description: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub images: MovieImages,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    media_type: crate::StrictValue,
}

#[async_trait::async_trait]
impl Streams for Movie {
    async fn streams(&self) -> Result<VideoStream> {
        let endpoint = format!(
            "https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams",
            self.executor.details.bucket, self.id
        );
        let builder = self
            .executor
            .client
            .get(endpoint)
            .query(&self.executor.media_query());

        self.executor.request(builder).await
    }
}
