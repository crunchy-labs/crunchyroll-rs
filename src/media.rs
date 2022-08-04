use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use crate::common::Image;
use crate::error::Result;
use crate::{Crunchyroll, FromId, Locale};

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct EpisodeImages {
    pub thumbnail: Vec<Vec<Image>>
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Episode<'a> {
    #[serde(skip)]
    #[serde(default = "Crunchyroll::default_for_struct")]
    crunchy: Option<&'a Crunchyroll>,

    pub id: String,
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
    // TODO: impl playback
    pub playback: String,

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
    media_type: crate::StrictValue,
}

#[async_trait::async_trait]
impl<'a> FromId<'a> for Episode<'a> {
    async fn from_id(crunchy: &'a Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/episodes/{}", crunchy.config.bucket, id);
        let builder = crunchy.client
            .get(endpoint)
            .query(&crunchy.media_query());

        let mut episode: Episode = crunchy.request(builder)
            .await?;
        episode.crunchy = Some(crunchy);
        Ok(episode)
    }
}

type MovieImages = EpisodeImages;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
pub struct Movie<'a> {
    #[serde(skip)]
    #[serde(default = "Crunchyroll::default_for_struct")]
    crunchy: Option<&'a Crunchyroll>,

    pub id: String,
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
    // TODO: impl playback
    pub playback: String,

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
impl<'a> FromId<'a> for Movie<'a> {
    async fn from_id(crunchy: &'a Crunchyroll, id: String) -> Result<Self> {
        let endpoint = format!("https://beta-api.crunchyroll.com/cms/v2/{}/movies/{}", crunchy.config.bucket, id);
        let builder = crunchy.client
            .get(endpoint)
            .query(&crunchy.media_query());

        let mut movie: Movie = crunchy.request(builder)
            .await?;
        movie.crunchy = Some(crunchy);
        Ok(movie)
    }
}
