use crate::common::Request;
use crate::crunchyroll::Executor;
use crate::media::anime::util::{fix_empty_episode_versions, fix_empty_season_versions};
use crate::media::util::request_media;
use crate::media::Media;
use crate::{Crunchyroll, Episode, Locale, Result, Series};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SeasonVersionRestrictionWindow {
    pub audio_locale: Locale,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub watch_start: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub watch_end: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub list_start: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub list_end: DateTime<Utc>,

    pub level: Vec<String>,
    pub geo: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SeasonVersion {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    #[serde(rename = "guid")]
    pub id: String,

    pub audio_locale: Locale,

    pub original: bool,
    #[serde(default)]
    pub restriction_windows: Vec<SeasonVersionRestrictionWindow>,

    #[cfg(feature = "__test_strict")]
    pub(crate) variant: crate::StrictValue,
}

impl SeasonVersion {
    /// Requests an actual [`Season`] from this version.
    pub async fn season(&self) -> Result<Season> {
        Season::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            &self.id,
        )
        .await
    }
}

/// Metadata for a season.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault)]
#[serde(remote = "Self")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Season {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub series_id: String,
    pub channel_id: String,
    #[serde(default)]
    pub identifier: String,

    pub title: String,
    pub slug_title: String,
    pub description: String,

    pub season_number: u32,
    pub season_sequence_number: u32,

    pub number_of_episodes: u32,

    pub is_complete: bool,

    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub season_tags: Vec<String>,
    /// Descriptors about the season episodes' content, e.g. 'Violence' or 'Sexualized Imagery'.
    #[serde(default)]
    pub content_descriptors: Vec<String>,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    #[serde(skip_serializing)]
    audio_locale: Option<Locale>,
    /// Most of the time, like 99%, this contains only one locale. But sometimes Crunchyroll does
    /// weird stuff and marks a season which clearly has only one locale with two locales. See
    /// [this](https://github.com/crunchy-labs/crunchy-cli/issues/81#issuecomment-1351813787) issue
    /// comment for an example.
    pub audio_locales: Vec<Locale>,
    pub subtitle_locales: Vec<Locale>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    /// If the season is not available this might contain some information why.
    pub availability_notes: String,

    /// All versions of this season (same season but each entry has a different language).
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    pub versions: Vec<SeasonVersion>,

    #[cfg(feature = "__test_strict")]
    // currently empty (on all of my tests) but its might be filled in the future
    images: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    seo_title: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    seo_description: Option<crate::StrictValue>,
}

impl Season {
    /// Returns the series the season belongs to.
    pub async fn series(&self) -> Result<Series> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/series/{}",
            self.series_id
        );
        Ok(request_media(self.executor.clone(), endpoint)
            .await?
            .remove(0))
    }

    /// Returns all episodes of this season.
    pub async fn episodes(&self) -> Result<Vec<Episode>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/content/v2/cms/seasons/{}/episodes",
            self.id
        );
        let mut episodes: Vec<Episode> = request_media(self.executor.clone(), endpoint).await?;
        for episode in &mut episodes {
            fix_empty_episode_versions(episode);
        }
        Ok(episodes)
    }

    /// Show in which audios this [`Season`] is also available.
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn available_versions(&mut self) -> Result<Vec<Locale>> {
        Ok(self
            .versions
            .iter()
            .map(|v| v.audio_locale.clone())
            .collect())
    }

    /// Get the versions of this [`Season`] which have the specified audio locale(s). Use [`Season::available_versions`] to see all supported locale.
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn version(&mut self, audio_locales: Vec<Locale>) -> Result<Vec<Season>> {
        let mut result = vec![];
        for version in &self.versions {
            if audio_locales.contains(&version.audio_locale) {
                result.push(version.season().await?)
            }
        }
        Ok(result)
    }

    /// Get all available other versions (same [`Season`] but different audio locale) for this [`Season`].
    #[deprecated(since = "0.11.4", note = "Use the `.versions` field directly")]
    pub async fn versions(&mut self) -> Result<Vec<Season>> {
        let mut result = vec![];
        for version in &self.versions {
            result.push(version.season().await?)
        }
        Ok(result)
    }
}

#[async_trait::async_trait]
impl Media for Season {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        let mut season: Season = request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/seasons/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0);
        fix_empty_season_versions(&mut season);
        Ok(season)
    }

    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
        for version in &mut self.versions {
            version.__set_executor(self.executor.clone()).await
        }
    }

    async fn __apply_fixes(&mut self) {
        if let Some(audio_locale) = &self.audio_locale {
            self.audio_locales.push(audio_locale.clone());
            crate::media::anime::util::real_dedup_vec(&mut self.audio_locales);
        }
    }

    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_experimental_stabilizations(&mut self) {
        if self.executor.fixes.locale_name_parsing {
            self.audio_locales = vec![crate::media::anime::util::parse_locale_from_slug_title(
                &self.slug_title,
            )]
        }
        if self.executor.fixes.season_number {
            let mut split = self.identifier.splitn(2, '|');
            let (_, season) = (
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
