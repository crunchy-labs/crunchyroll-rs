use crate::common::{Image, Request};
use crate::crunchyroll::Executor;
use crate::media::util::request_media;
use crate::media::Media;
use crate::{Crunchyroll, Locale, Result, Season, Series};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct EpisodeVersion {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    #[serde(rename = "guid")]
    pub id: String,
    #[serde(rename = "media_guid")]
    pub media_id: String,
    #[serde(rename = "season_guid")]
    pub season_id: String,

    pub audio_locale: Locale,

    pub is_premium_only: bool,
    pub original: bool,

    #[cfg(feature = "__test_strict")]
    variant: crate::StrictValue,
}

impl EpisodeVersion {
    /// Requests an actual [`Episode`] from this version.
    pub async fn episode(&self) -> Result<Episode> {
        Episode::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            &self.id,
        )
        .await
    }
}

/// Metadata for an episode.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Episode {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(alias = "streams_link")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_streams_link")]
    pub stream_id: String,
    pub channel_id: String,
    pub identifier: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub description: String,

    // both missing if the episode is the last one in its season unpopulated
    #[serde(default)]
    pub next_episode_id: String,
    #[serde(default)]
    pub next_episode_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    #[serde(default)]
    pub season_tags: Vec<String>,
    pub season_sequence_number: f32,

    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    // probably empty
    #[serde(default)]
    pub production_episode_id: String,

    /// Usually the same as [`Episode::episode_number`], just as string.
    pub episode: String,
    /// The episode number may be null. In most of the cases this is when the episode is a special,
    /// like 0.5. Consider using [`Episode::sequence_number`] instead as this is always populated.
    pub episode_number: Option<u32>,
    /// Usually also the same as [`Episode::episode_number`]. If the episode number is null (which
    /// occurs for the first AOT episode, which is a preview, for example) this might be a floating
    /// number like 0.5.
    pub sequence_number: f32,

    pub season_number: u32,
    pub season_display_number: String,

    pub audio_locale: Locale,
    /// Only populated if [`Episode`] got generated via [`Season::episodes`].
    pub recent_audio_locale: Option<Locale>,
    pub subtitle_locales: Vec<Locale>,

    /// Descriptors about the episode content, e.g. 'Violence' or 'Sexualized Imagery'.
    #[serde(default)]
    pub content_descriptors: Vec<String>,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[serde(serialize_with = "crate::internal::serde::serialize_duration_to_millis")]
    #[default(Duration::try_milliseconds(0).unwrap())]
    pub duration: Duration,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub episode_air_date: DateTime<Utc>,
    /// The same as episode_air_date as far as I can see.
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

    #[serde(deserialize_with = "crate::internal::serde::deserialize_thumbnail_image")]
    pub images: Vec<Image>,

    pub is_dubbed: bool,
    pub is_subbed: bool,

    pub is_premium_only: bool,
    pub is_clip: bool,

    pub is_mature: bool,
    pub maturity_ratings: Vec<String>,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub closed_captions_available: bool,

    pub eligible_region: String,

    /// Alternative versions of this episode (same episode but other language).
    pub versions: Vec<EpisodeVersion>,

    #[cfg(feature = "__test_strict")]
    media_type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    external_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    linked_resource_key: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    promo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    search_metadata: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    #[serde(rename = "type")]
    _type: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    tenant_categories: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    listing_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    hd_flag: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    recent_variant: Option<crate::StrictValue>,
}

impl Episode {
    /// Returns the series the episode belongs to.
    pub async fn series(&self) -> Result<Series> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}",
            self.series_id
        );
        Ok(request_media(self.executor.clone(), endpoint)
            .await?
            .remove(0))
    }

    /// Returns the season the episode belongs to.
    pub async fn season(&self) -> Result<Season> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/seasons/{}",
            self.season_id
        );
        Ok(request_media(self.executor.clone(), endpoint)
            .await?
            .remove(0))
    }

    /// Show in which audios this [`Episode`] is also available.
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn available_versions(&mut self) -> Result<Vec<Locale>> {
        Ok(self
            .versions
            .iter()
            .map(|v| v.audio_locale.clone())
            .collect())
    }

    /// Get the versions of this [`Episode`] which have the specified audio locale(s). Use [`Episode::available_versions`] to see all supported locale.
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn version(&mut self, audio_locales: Vec<Locale>) -> Result<Vec<Episode>> {
        let mut result = vec![];
        for version in &self.versions {
            if audio_locales.contains(&version.audio_locale) {
                result.push(version.episode().await?)
            }
        }
        Ok(result)
    }

    /// Get all available other versions (same [`Episode`] but different audio locale) for this [`Episode`].
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn versions(&mut self) -> Result<Vec<Episode>> {
        let mut result = vec![];
        for version in &self.versions {
            result.push(version.episode().await?)
        }
        Ok(result)
    }
}

#[async_trait::async_trait]
impl Media for Episode {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        Ok(request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/episodes/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0))
    }

    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
        for version in &mut self.versions {
            version.__set_executor(self.executor.clone()).await
        }
    }

    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_experimental_stabilizations(&mut self) {
        if self.executor.fixes.locale_name_parsing {
            self.audio_locale =
                crate::media::anime::util::parse_locale_from_slug_title(&self.season_slug_title)
        }
        if self.executor.fixes.season_number {
            let mut split = self.identifier.splitn(3, '|');
            let (_, season, _) = (
                split.next().unwrap_or_default(),
                split.next().unwrap_or_default(),
                split.next().unwrap_or_default(),
            );

            if let Some(maybe_number) = season.strip_prefix('S') {
                let mut num_string = String::new();
                for c in maybe_number.chars() {
                    if c.to_string().parse::<u32>().is_err() {
                        break;
                    }
                    num_string.push(c)
                }
                if !num_string.is_empty() {
                    self.season_number = num_string.parse::<u32>().unwrap()
                }
            }
        }
    }
}
