use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

use crate::common::Image;
use crate::media::MediaImages;
use crate::{Episode, Executor, Locale, Media, Movie, Request, Season};

#[derive(Debug, Deserialize, Default)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct OldMediaImages {
    thumbnail: Option<Vec<Vec<Image>>>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct OldEpisode {
    #[serde(skip)]
    executor: Arc<Executor>,

    id: String,
    #[serde(rename = "__links__")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id")]
    stream_id: String,
    #[serde(rename = "playback")]
    playback_url: String,
    channel_id: String,
    // whatever this is
    production_episode_id: String,
    // not really needed ig
    listing_id: String,

    slug: String,
    title: String,
    slug_title: String,
    seo_title: String,
    description: String,
    seo_description: String,

    series_id: String,
    series_title: String,
    series_slug_title: String,

    season_id: String,
    season_title: String,
    season_slug_title: String,
    season_number: u32,

    // usually the same as episode_number, just as string
    episode: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    episode_number: u32,
    // usually also the same as episode_number. if the episode number is null (which occurs for the
    // first AOT episode, which is a preview, for example) this might be a floating number like 0.5
    sequence_number: f32,
    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    episode_air_date: DateTime<Utc>,
    // the same as episode_air_date as far as I can see
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    upload_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    premium_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    availability_starts: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    availability_ends: DateTime<Utc>,

    is_subbed: bool,
    is_dubbed: bool,
    closed_captions_available: bool,

    audio_locale: Locale,
    subtitle_locales: Vec<Locale>,

    #[serde(default)]
    // the api result simply does not contain this field if the episode is the last of its season.
    // classic crunchyroll moment
    next_episode_id: Option<String>,
    #[serde(default)]
    // the api result simply does not contain this field if the episode is the last of its season.
    // classic crunchyroll moment
    next_episode_title: Option<String>,

    season_tags: Vec<String>,

    images: OldMediaImages,

    hd_flag: bool,
    is_clip: bool,
    is_premium_only: bool,

    maturity_ratings: Vec<String>,
    is_mature: bool,
    mature_blocked: bool,

    available_offline: bool,
    availability_notes: String,

    eligible_region: String,

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

#[allow(clippy::from_over_into)]
impl Into<Media<Episode>> for OldEpisode {
    fn into(self) -> Media<Episode> {
        Media {
            executor: self.executor,
            id: self.id,
            stream_id: Some(self.stream_id),
            playback_url: Some(self.playback_url),
            external_id: "".to_string(),
            channel_id: self.channel_id,
            slug: self.slug,
            title: self.title,
            slug_title: self.slug_title.clone(),
            promo_title: self.slug_title,
            description: self.description.clone(),
            promo_description: self.description,
            metadata: Episode {
                series_id: self.series_id,
                series_title: self.series_title,
                series_slug_title: self.series_slug_title,
                season_id: self.season_id,
                season_title: self.season_title,
                season_slug_title: self.season_slug_title,
                season_number: self.season_number,
                episode: self.episode,
                episode_number: self.episode_number,
                sequence_number: self.sequence_number,
                duration: self.duration,
                episode_air_date: self.episode_air_date,
                upload_date: self.upload_date,
                free_available_date: self.free_available_date,
                premium_available_date: self.premium_available_date,
                availability_starts: self.availability_starts,
                availability_ends: self.availability_ends,
                is_subbed: self.is_subbed,
                is_dubbed: self.is_dubbed,
                closed_captions_available: self.closed_captions_available,
                audio_locale: self.audio_locale,
                subtitle_locales: self.subtitle_locales,
                is_clip: self.is_clip,
                is_premium_only: self.is_premium_only,
                categories: vec![],
                maturity_ratings: self.maturity_ratings,
                is_mature: self.is_mature,
                mature_blocked: self.mature_blocked,
                available_offline: self.available_offline,
                availability_notes: self.availability_notes,
                eligible_region: self.eligible_region,
                #[cfg(feature = "__test_strict")]
                extended_maturity_rating: self.extended_maturity_rating,
                #[cfg(feature = "__test_strict")]
                available_date: self.available_date,
                #[cfg(feature = "__test_strict")]
                premium_date: self.premium_date,
                #[cfg(feature = "__test_strict")]
                versions: self.versions,
                #[cfg(feature = "__test_strict")]
                identifier: self.identifier,
            },
            search_metadata: None,
            images: Some(MediaImages {
                thumbnail: self.images.thumbnail,
                poster_tall: None,
                poster_wide: None,
                promo_image: None,
            }),
            #[cfg(feature = "__test_strict")]
            collection_type: Default::default(),
            #[cfg(feature = "__test_strict")]
            new: None,
            #[cfg(feature = "__test_strict")]
            new_content: None,
            #[cfg(feature = "__test_strict")]
            last_public: None,
            #[cfg(feature = "__test_strict")]
            linked_resource_key: Default::default(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct OldSeason {
    #[serde(skip)]
    executor: Arc<Executor>,

    id: String,
    series_id: String,
    channel_id: String,

    title: String,
    slug_title: String,
    seo_title: String,
    description: String,
    seo_description: String,

    season_number: u32,

    is_complete: bool,

    keywords: Vec<String>,
    season_tags: Vec<String>,

    is_subbed: bool,
    is_dubbed: bool,
    is_simulcast: bool,
    audio_locale: Locale,
    audio_locales: Vec<Locale>,
    subtitle_locales: Vec<Locale>,

    maturity_ratings: Vec<String>,
    is_mature: bool,
    mature_blocked: bool,

    availability_notes: String,

    #[cfg(feature = "__test_strict")]
    // currently empty (on all of my tests) but its might be filled in the future
    images: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_sequence_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    identifier: crate::StrictValue,
}

#[allow(clippy::from_over_into)]
impl Into<Media<Season>> for OldSeason {
    fn into(mut self) -> Media<Season> {
        if self.audio_locale != Locale::default() {
            self.audio_locales.push(self.audio_locale);
            self.audio_locales.dedup()
        }
        Media {
            executor: self.executor,
            id: self.id,
            stream_id: None,
            playback_url: None,
            external_id: "".to_string(),
            channel_id: self.channel_id,
            slug: self.slug_title.clone(),
            title: self.title,
            slug_title: self.slug_title.clone(),
            promo_title: self.slug_title,
            description: self.description.clone(),
            promo_description: self.description,
            metadata: Season {
                audio_locales: self.audio_locales,
                subtitle_locales: self.subtitle_locales,
                season_number: self.season_number,
                maturity_ratings: self.maturity_ratings,
                is_mature: self.is_mature,
                mature_blocked: self.mature_blocked,
                #[cfg(feature = "__test_strict")]
                season_display_number: self.season_display_number,
                #[cfg(feature = "__test_strict")]
                season_sequence_number: self.season_sequence_number,
                #[cfg(feature = "__test_strict")]
                extended_maturity_rating: self.extended_maturity_rating,
                #[cfg(feature = "__test_strict")]
                versions: self.versions,
                #[cfg(feature = "__test_strict")]
                identifier: self.identifier,
            },
            search_metadata: None,
            images: None,
            #[cfg(feature = "__test_strict")]
            collection_type: Default::default(),
            #[cfg(feature = "__test_strict")]
            new: None,
            #[cfg(feature = "__test_strict")]
            new_content: None,
            #[cfg(feature = "__test_strict")]
            last_public: None,
            #[cfg(feature = "__test_strict")]
            linked_resource_key: Default::default(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct OldMovie {
    #[serde(skip)]
    executor: Arc<Executor>,

    id: String,
    #[serde(rename = "playback")]
    playback_url: String,
    channel_id: String,
    // id of corresponding movie_listing object
    listing_id: String,

    slug: String,
    title: String,
    movie_listing_title: String,
    slug_title: String,
    description: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    premium_available_date: DateTime<Utc>,

    is_subbed: bool,
    is_dubbed: bool,
    closed_captions_available: bool,

    images: OldMediaImages,

    is_premium_only: bool,

    maturity_ratings: Vec<String>,
    is_mature: bool,
    mature_blocked: bool,

    available_offline: bool,
    availability_notes: String,

    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    media_type: crate::StrictValue,
}

#[allow(clippy::from_over_into)]
impl Into<Media<Movie>> for OldMovie {
    fn into(self) -> Media<Movie> {
        Media {
            executor: self.executor,
            id: self.id.clone(),
            stream_id: Some(self.id.clone()),
            playback_url: Some(self.playback_url),
            external_id: "".to_string(),
            channel_id: self.channel_id,
            slug: self.slug,
            title: self.title,
            slug_title: self.slug_title.clone(),
            promo_title: self.slug_title,
            description: self.description.clone(),
            promo_description: self.description,
            metadata: Movie {
                movie_listing_id: self.id,
                movie_listing_title: self.movie_listing_title.clone(),
                movie_listing_slug_title: self.movie_listing_title,
                duration: self.duration,
                is_subbed: self.is_subbed,
                is_dubbed: self.is_dubbed,
                closed_captions_available: self.closed_captions_available,
                is_premium_only: self.is_premium_only,
                maturity_ratings: self.maturity_ratings,
                is_mature: self.is_mature,
                mature_blocked: self.mature_blocked,
                available_offline: self.available_offline,
                availability_notes: self.availability_notes,
                #[cfg(feature = "__test_strict")]
                extended_maturity_rating: self.extended_maturity_rating,
            },
            search_metadata: None,
            images: Some(MediaImages {
                thumbnail: self.images.thumbnail,
                poster_tall: None,
                poster_wide: None,
                promo_image: None,
            }),
            #[cfg(feature = "__test_strict")]
            collection_type: Default::default(),
            #[cfg(feature = "__test_strict")]
            new: None,
            #[cfg(feature = "__test_strict")]
            new_content: None,
            #[cfg(feature = "__test_strict")]
            last_public: None,
            #[cfg(feature = "__test_strict")]
            linked_resource_key: Default::default(),
        }
    }
}
