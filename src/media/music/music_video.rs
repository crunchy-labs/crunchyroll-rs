use crate::crunchyroll::Executor;
use crate::media::music::util::availability_object_to_keys;
use crate::media::util::request_media;
use crate::media::{ArtistPreview, ArtistsPreviewList, Media, MusicGenre, ThumbnailImages};
use crate::{Crunchyroll, MediaCollection, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use std::sync::Arc;

/// Metadata for a music video.
#[derive(Clone, Debug, Deserialize, Serialize, Request, smart_default::SmartDefault)]
#[request(executor(artist, artists))]
#[serde(rename_all = "camelCase")]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MusicVideo {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(alias = "streams_link")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_streams_link")]
    pub stream_id: String,
    /// Ids of related anime series. Use [`crate::Series::from_id`] to get series from it.
    pub anime_ids: Vec<String>,

    pub slug: String,
    pub title: String,
    pub description: String,

    pub sequence_number: f32,

    pub artist: ArtistPreview,
    pub artists: ArtistsPreviewList,
    pub display_artist_name: String,
    pub display_artist_name_required: bool,

    pub licensor: String,
    pub copyright: String,

    pub images: ThumbnailImages,
    pub genres: Vec<MusicGenre>,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub updated_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub publish_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub original_release: DateTime<Utc>,

    #[serde(alias = "durationMs")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[serde(serialize_with = "crate::internal::serde::serialize_duration_to_millis")]
    #[default(Duration::try_milliseconds(0).unwrap())]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_starts: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub availability_ends: DateTime<Utc>,

    pub is_premium_only: bool,
    pub is_public: bool,
    pub ready_to_publish: bool,

    pub is_mature: bool,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_object_to_array")]
    pub maturity_ratings: Vec<String>,
    pub mature_blocked: bool,

    /// Yea a hash. Md5. For what every reason.
    pub hash: String,

    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    type_: crate::StrictValue,
}

impl<'de> Deserialize<'de> for MusicVideo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut as_map = serde_json::Map::deserialize(deserializer)?;

        availability_object_to_keys(&mut as_map).map_err(|e| Error::custom(e.to_string()))?;

        MusicVideo::deserialize(
            serde_json::to_value(as_map)
                .map_err(|e| Error::custom(e.to_string()))?
                .into_deserializer(),
        )
        .map_err(|e| Error::custom(e.to_string()))
    }
}

impl MusicVideo {
    /// Return all related anime with this music video.
    pub async fn related_anime(&self) -> Result<Vec<MediaCollection>> {
        let mut media = vec![];

        for id in &self.anime_ids {
            media.push(
                Crunchyroll::media_collection_from_id(
                    &Crunchyroll {
                        executor: self.executor.clone(),
                    },
                    id,
                )
                .await?,
            )
        }

        Ok(media)
    }
}

impl Media for MusicVideo {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/music/music_videos/{}",
            id.as_ref()
        );
        Ok(request_media(crunchyroll.executor.clone(), endpoint)
            .await?
            .remove(0))
    }

    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor
    }
}
