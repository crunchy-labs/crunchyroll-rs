use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::de::Error;
use serde::Deserialize;

use crate::{Crunchyroll, Executor, FromId, Locale, PlaybackStream, VideoStream};
use crate::common::{Image, Playback, Request, Streams};
use crate::error::Result;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct EpisodeImages {
    pub thumbnail: Vec<Vec<Image>>
}

/// This struct represents a Crunchyroll episode.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Episode {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(deserialize_with = "stream_id")]
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
    #[serde(deserialize_with = "crate::internal::serde::millis_to_duration")]
    #[cfg_attr(not(feature = "__test_strict"), default(Duration::milliseconds(0)))]
    pub duration: Duration,

    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub episode_air_date: DateTime<Utc>,
    // the same as episode_air_date as far as I can see
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub upload_date: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub free_available_date: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub premium_available_date: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub availability_starts: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub availability_ends: DateTime<Utc>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,
    // would be very useful, but is (currently) always empty
    pub audio_locale: String,
    pub subtitle_locales: Vec<Locale>,

    pub next_episode_id: String,
    pub next_episode_title: String,

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
    fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor
    }

    #[cfg(feature = "__test_strict")]
    fn not_clean_fields() -> Vec<String> {
        vec![
            "__links__".into()
        ]
    }
}

#[async_trait::async_trait]
impl FromId for Episode {
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self> {
        let executor = crunchy.executor.clone();

        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/episodes/{}", executor.config.bucket, id);
        let builder = executor.client
            .get(endpoint)
            .query(&executor.media_query());

        executor.request(builder).await
    }
}

#[async_trait::async_trait]
impl Playback for Episode {
    async fn playback(&self) -> Result<PlaybackStream> {
        self.executor.request(self.executor.client.get(self.playback_id.clone())).await
    }
}

#[async_trait::async_trait]
impl Streams for Episode {
    async fn streams(&self) -> Result<VideoStream> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams", self.executor.config.bucket, self.stream_id);
        let builder = self.executor.client
            .get(endpoint)
            .query(&self.executor.media_query());

        self.executor.request(builder).await
    }
}

type MovieImages = EpisodeImages;

/// This struct represents a Crunchyroll movie.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Movie {
    #[serde(skip)]
    executor: Arc<Executor>,

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
    #[serde(deserialize_with = "crate::internal::serde::millis_to_duration")]
    #[cfg_attr(not(feature = "__test_strict"), default(Duration::milliseconds(0)))]
    pub duration: Duration,

    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
    pub free_available_date: DateTime<Utc>,
    #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
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

impl Request for Movie {
    fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor
    }
}

#[async_trait::async_trait]
impl FromId for Movie {
    async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Self> {
        let executor = crunchy.executor.clone();

        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/movies/{}", executor.config.bucket, id);
        let builder = executor.client
            .get(endpoint)
            .query(&executor.media_query());

        executor.request(builder).await
    }
}


#[async_trait::async_trait]
impl Playback for Movie {
    async fn playback(&self) -> Result<PlaybackStream> {
        self.executor.request(self.executor.client.get(self.playback_id.clone())).await
    }
}

#[async_trait::async_trait]
impl Streams for Movie {
    async fn streams(&self) -> Result<VideoStream> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/videos/{}/streams", self.executor.config.bucket, self.id);
        let builder = self.executor.client
            .get(endpoint)
            .query(&self.executor.media_query());

        self.executor.request(builder).await
    }
}

fn stream_id<'de, D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<String, D::Error> {
    #[derive(Deserialize)]
    struct StreamHref {
        href: String
    }
    #[derive(Deserialize)]
    struct Streams {
        streams: StreamHref
    }

    let streams: Streams = serde_json::from_value(serde_json::Value::deserialize(deserializer)?)
        .map_err(|e| Error::custom(e.to_string()))?;

    let mut split_streams = streams.streams.href.split('/').map(|s| s.to_string()).collect::<Vec<String>>();
    split_streams.reverse();
    if let Some(stream_id) = split_streams.get(1) {
        Ok(stream_id.clone())
    } else {
        Err(Error::custom("cannot extract stream id"))
    }
}
