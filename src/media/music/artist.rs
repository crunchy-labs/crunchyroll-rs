use crate::common::V2BulkResult;
use crate::crunchyroll::Executor;
use crate::media::music::concert::Concert;
use crate::media::util::request_media;
use crate::media::{Genre, MusicVideo, PosterImages};
use crate::{Crunchyroll, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct ArtistPreview {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,

    pub slug: String,
    pub name: String,
}

impl ArtistPreview {
    pub async fn artist(&self) -> Result<Artist> {
        Artist::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            &self.id,
        )
        .await
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Request, smart_default::SmartDefault)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Artist {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "concerts")]
    pub concert_ids: Vec<String>,
    #[serde(rename = "videos")]
    pub video_ids: Vec<String>,

    pub slug: String,
    pub name: String,
    pub description: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub updated_at: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub publish_date: DateTime<Utc>,

    #[serde(alias = "totalConcertDurationMs")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub total_concert_duration: Duration,
    #[serde(alias = "totalVideoDurationMs")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub total_video_duration: Duration,

    pub images: PosterImages,
    pub genres: Vec<Genre>,

    pub is_public: bool,
    pub ready_to_publish: bool,

    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    type_: crate::StrictValue,
}

impl Artist {
    pub async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/music/artists/{}",
            id.as_ref()
        );
        Ok(request_media(crunchyroll.executor.clone(), endpoint)
            .await?
            .remove(0))
    }

    /// Return all concerts of this artist.
    pub async fn concerts(&self) -> Result<Vec<Concert>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/music/artists/{}/concerts",
            &self.id
        );
        Ok(self
            .executor
            .get(endpoint)
            .apply_locale_query()
            .request::<V2BulkResult<Concert>>()
            .await?
            .data)
    }

    /// Return all music videos of this artist.
    pub async fn music_videos(&self) -> Result<Vec<MusicVideo>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/music/artists/{}/music_videos",
            &self.id
        );
        Ok(self
            .executor
            .get(endpoint)
            .apply_locale_query()
            .request::<V2BulkResult<MusicVideo>>()
            .await?
            .data)
    }
}
