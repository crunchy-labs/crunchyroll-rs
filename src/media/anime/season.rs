use crate::crunchyroll::Executor;
use crate::media::anime::util::{parse_locale_from_slug_title, real_dedup_vec};
use crate::media::util::request_media;
use crate::media::Media;
use crate::{Crunchyroll, Episode, Locale, Result};
use serde::Deserialize;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct SeasonVersion {
    #[serde(rename = "guid")]
    pub(crate) id: String,

    pub(crate) audio_locale: Locale,

    pub(crate) original: bool,

    pub(crate) variant: String,
}

/// Metadata for a season.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
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

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
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

    #[serde(default)]
    pub(crate) versions: Option<Vec<SeasonVersion>>,

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
    pub async fn series(&self) -> Result<Season> {
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
        request_media(self.executor.clone(), endpoint).await
    }
}

#[async_trait::async_trait]
impl Media for Season {
    async fn from_id(crunchyroll: &Crunchyroll, id: impl AsRef<str> + Send) -> Result<Self> {
        Ok(request_media(
            crunchyroll.executor.clone(),
            format!(
                "https://www.crunchyroll.com/content/v2/cms/seasons/{}",
                id.as_ref()
            ),
        )
        .await?
        .remove(0))
    }

    async fn __apply_fixes(&mut self) {
        if let Some(audio_locale) = &self.audio_locale {
            self.audio_locales.push(audio_locale.clone());
            real_dedup_vec(&mut self.audio_locales);
        }
    }

    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_experimental_stabilizations(&mut self) {
        if self.executor.fixes.locale_name_parsing {
            self.audio_locales = vec![parse_locale_from_slug_title(&self.slug_title)]
        }
        if self.executor.fixes.season_number {
            let mut split = self.identifier.splitn(2, '|');
            let (_, season) = (
                split.next().unwrap_or_default(),
                split.next().unwrap_or_default(),
            );

            if let Ok(season_num) = season.trim_start_matches('S').parse() {
                self.season_number = season_num
            }
        }
    }
}
