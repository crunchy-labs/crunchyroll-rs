use crate::crunchyroll::Executor;
use crate::media::music::util::availability_object_to_keys;
use crate::media::util::request_media;
use crate::media::{ArtistPreview, Genre, Media, ThumbnailImages};
use crate::{Crunchyroll, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Request, smart_default::SmartDefault)]
#[request(executor(artist))]
#[serde(rename_all = "camelCase")]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Concert {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(alias = "streams_link")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_streams_link")]
    pub stream_id: String,

    pub slug: String,
    pub title: String,
    pub description: String,

    pub sequence_number: f32,

    pub artist: ArtistPreview,
    pub licensor: String,
    pub copyright: String,

    pub images: ThumbnailImages,
    pub genres: Vec<Genre>,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub updated_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub publish_date: DateTime<Utc>,

    #[serde(alias = "durationMs")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub original_release: DateTime<Utc>,

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

impl<'de> Deserialize<'de> for Concert {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut as_map = serde_json::Map::deserialize(deserializer)?;

        availability_object_to_keys(&mut as_map).map_err(|e| Error::custom(e.to_string()))?;

        Concert::deserialize(
            serde_json::to_value(as_map)
                .map_err(|e| Error::custom(e.to_string()))?
                .into_deserializer(),
        )
        .map_err(|e| Error::custom(e.to_string()))
    }
}

#[async_trait::async_trait]
impl Media for Concert {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/music/concerts/{}",
            id.as_ref()
        );
        Ok(request_media(crunchyroll.executor.clone(), endpoint)
            .await?
            .remove(0))
    }
}
