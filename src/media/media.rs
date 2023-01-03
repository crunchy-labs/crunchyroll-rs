use crate::categories::Category;
use crate::common::{BulkResult, Image};
use crate::error::{CrunchyrollError, CrunchyrollErrorContext};
use crate::media::old_media::{OldEpisode, OldMovie, OldSeason};
use crate::media::{PlaybackStream, VideoStream};
use crate::{options, Crunchyroll, Executor, Locale, Request, Result};
use chrono::{DateTime, Duration, Utc};
use serde::de::{DeserializeOwned, IntoDeserializer};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::sync::Arc;

#[async_trait::async_trait(?Send)]
pub trait Video: Default + DeserializeOwned + Request {
    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_fixes(_: Arc<Executor>, _: &mut Media<Self>) {}
}

#[cfg(feature = "experimental-stabilizations")]
pub(crate) fn parse_locale_from_series_title<S: AsRef<str>>(title: S) -> Locale {
    lazy_static::lazy_static! {
        static ref SERIES_LOCALE_REGEX: regex::Regex = regex::Regex::new(r".*\((?P<locale>\S+)(\sDub)?\)$").unwrap();
    }

    if let Some(capture) = SERIES_LOCALE_REGEX.captures(title.as_ref()) {
        match capture.name("locale").unwrap().as_str() {
            "Castilian" => Locale::es_ES,
            "English" => Locale::en_US,
            "English-IN" => Locale::en_IN,
            "French" => Locale::fr_FR,
            "German" => Locale::de_DE,
            "Hindi" => Locale::hi_IN,
            "Italian" => Locale::it_IT,
            "Portuguese" => Locale::pt_BR,
            "Russian" => Locale::ru_RU,
            "Spanish" => Locale::es_419,
            _ => Locale::ja_JP,
        }
    } else {
        Locale::ja_JP
    }
}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SearchMetadata {
    /// [`None`] if queried by [`crate::Crunchyroll::query`].
    pub last_public: Option<DateTime<Utc>>,
    /// [`None`] if queried by [`crate::Crunchyroll::query`].
    pub rank: Option<u32>,

    pub score: f64,
    /// [`None`] if not queried by [`crate::Media<Series>::similar`] or
    /// [`crate::Media<MovieListing>::similar`].
    pub popularity_score: Option<f64>,
}

/// Metadata for a [`Media`] series.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Series {
    pub extended_description: String,

    pub series_launch_year: Option<u32>,

    pub episode_count: u32,
    pub season_count: u32,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub is_simulcast: bool,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub audio_locales: Vec<Locale>,
    /// Might be empty. Some series have this field populated with locales, others not.
    pub subtitle_locales: Vec<Locale>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl Video for Series {
    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_fixes(executor: Arc<Executor>, media: &mut Media<Self>) {
        if executor.fixes.locale_name_parsing {
            if let Ok(seasons) = media.seasons().await {
                let mut locales = seasons
                    .into_iter()
                    .flat_map(|s| s.metadata.audio_locales)
                    .collect::<Vec<Locale>>();
                locales.dedup();

                media.metadata.audio_locales = locales
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct SeasonProxy {
    audio_locale: Locale,
    audio_locales: Vec<Locale>,
    subtitle_locales: Vec<Locale>,

    #[serde(default)]
    season_number: u32,

    maturity_ratings: Vec<String>,
    is_mature: bool,
    mature_blocked: bool,

    #[cfg(feature = "__test_strict")]
    pub(crate) season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) season_sequence_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) identifier: crate::StrictValue,
}

/// Metadata for a [`Media`] season.
/// The deserializing requires a proxy struct because the season json response has two similar
/// fields, `audio_locale` and `audio_locales`. Sometimes the first is populated, sometimes the
/// second and sometimes both. They're representing the season audio but why it needs two fields
/// for this, who knows. `audio_locale` is also a [`Vec`] of locales but, if populated, contains
/// always only one locale.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[serde(from = "SeasonProxy")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Season {
    /// Most of the time, like 99%, this contains only one locale. But sometimes Crunchyroll does
    /// weird stuff and marks a season which clearly has only one locale with two locales. See
    /// [this](https://github.com/crunchy-labs/crunchy-cli/issues/81#issuecomment-1351813787) issue
    /// comment for an example.
    pub audio_locales: Vec<Locale>,
    /// Sometimes populated, sometimes not. idk why. Crunchyroll.
    pub subtitle_locales: Vec<Locale>,

    /// Currently only populated if this season got generated by [`Media<Series>::seasons`].
    #[serde(default)]
    pub season_number: u32,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    #[cfg(feature = "__test_strict")]
    pub(crate) season_display_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) season_sequence_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) identifier: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl Video for Season {
    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_fixes(executor: Arc<Executor>, media: &mut Media<Self>) {
        if executor.fixes.locale_name_parsing {
            media.metadata.audio_locales = vec![parse_locale_from_series_title(&media.title)];
        }
    }
}

impl From<SeasonProxy> for Season {
    fn from(mut season_proxy: SeasonProxy) -> Self {
        if season_proxy.audio_locale != Locale::default() {
            season_proxy.audio_locales.push(season_proxy.audio_locale);
            season_proxy.audio_locales.dedup()
        }
        Self {
            audio_locales: season_proxy.audio_locales,
            subtitle_locales: season_proxy.subtitle_locales,
            season_number: season_proxy.season_number,
            maturity_ratings: season_proxy.maturity_ratings,
            is_mature: season_proxy.is_mature,
            mature_blocked: season_proxy.mature_blocked,
            #[cfg(feature = "__test_strict")]
            season_display_number: season_proxy.season_display_number,
            #[cfg(feature = "__test_strict")]
            season_sequence_number: season_proxy.season_sequence_number,
            #[cfg(feature = "__test_strict")]
            extended_maturity_rating: season_proxy.extended_maturity_rating,
            #[cfg(feature = "__test_strict")]
            versions: season_proxy.versions,
            #[cfg(feature = "__test_strict")]
            identifier: season_proxy.identifier,
        }
    }
}

/// Metadata for a [`Media`] episode.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Episode {
    pub series_id: String,
    pub series_title: String,
    pub series_slug_title: String,

    pub season_id: String,
    pub season_title: String,
    pub season_slug_title: String,
    pub season_number: u32,

    /// Usually the same as [`Episode::episode_number`], just as string.
    pub episode: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_maybe_null_to_default")]
    pub episode_number: u32,
    /// Usually also the same as [`Episode::episode_number`]. If the episode number is null (which
    /// occurs for the first AOT episode, which is a preview, for example) this might be a floating
    /// number like 0.5.
    pub sequence_number: f32,
    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
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

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub audio_locale: Locale,
    pub subtitle_locales: Vec<Locale>,

    pub is_clip: bool,
    pub is_premium_only: bool,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    pub eligible_region: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) premium_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) versions: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) identifier: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl Video for Episode {
    #[cfg(feature = "experimental-stabilizations")]
    async fn __apply_fixes(executor: Arc<Executor>, media: &mut Media<Self>) {
        if executor.fixes.locale_name_parsing {
            media.metadata.audio_locale =
                parse_locale_from_series_title(&media.metadata.series_title)
        }
    }
}

/// Metadata for a [`Media`] movie listing.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MovieListing {
    // wtf is this again
    pub first_movie_id: String,

    pub extended_description: String,

    pub movie_release_year: u32,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub subtitle_locales: Vec<Locale>,

    pub is_premium_only: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub free_available_date: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub premium_available_date: DateTime<Utc>,

    #[serde(default)]
    #[serde(rename = "tenant_categories")]
    pub categories: Vec<Category>,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) available_date: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) premium_date: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl Video for MovieListing {}

/// Metadata for a [`Media`] movie.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Movie {
    pub movie_listing_id: String,

    pub movie_listing_title: String,
    pub movie_listing_slug_title: String,

    #[serde(alias = "duration_ms")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_millis_to_duration")]
    #[default(Duration::milliseconds(0))]
    pub duration: Duration,

    pub is_subbed: bool,
    pub is_dubbed: bool,
    pub closed_captions_available: bool,

    pub is_premium_only: bool,

    pub maturity_ratings: Vec<String>,
    pub is_mature: bool,
    pub mature_blocked: bool,

    pub available_offline: bool,
    pub availability_notes: String,

    #[cfg(feature = "__test_strict")]
    pub(crate) extended_maturity_rating: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl Video for Movie {}

/// Collection of all media types. Useful in situations where [`Media`] can contain more than one
/// specific media.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MediaCollection {
    Series(Media<Series>),
    Season(Media<Season>),
    Episode(Media<Episode>),
    MovieListing(Media<MovieListing>),
    Movie(Media<Movie>),
}

impl Default for MediaCollection {
    fn default() -> Self {
        Self::Series(Media::default())
    }
}

#[async_trait::async_trait(?Send)]
impl Request for MediaCollection {
    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        match self {
            MediaCollection::Series(series) => series.__set_executor(executor).await,
            MediaCollection::Season(season) => season.__set_executor(executor).await,
            MediaCollection::Episode(episode) => episode.__set_executor(executor).await,
            MediaCollection::MovieListing(movie_listing) => {
                movie_listing.__set_executor(executor).await
            }
            MediaCollection::Movie(movie) => movie.__set_executor(executor).await,
        }
    }
}

impl<'de> Deserialize<'de> for MediaCollection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_map = serde_json::Map::deserialize(deserializer)?;

        let err_conv = |e: serde_json::Error| serde::de::Error::custom(e.to_string());

        if as_map.contains_key("series_metadata") {
            Ok(MediaCollection::Series(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("season_metadata") {
            Ok(MediaCollection::Season(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("episode_metadata") {
            Ok(MediaCollection::Episode(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_listing_metadata") {
            Ok(MediaCollection::MovieListing(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_metadata") {
            Ok(MediaCollection::Movie(
                Media::deserialize(Value::from(as_map).into_deserializer()).map_err(err_conv)?,
            ))
        } else {
            Err(serde::de::Error::custom(
                "no metadata were found".to_string(),
            ))
        }
    }
}

impl MediaCollection {
    pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<MediaCollection> {
        let endpoint = format!(
            "https://www.crunchyroll.com/cms/v2/{}/objects/{}",
            crunchy.executor.details.bucket, &id
        );
        let result: BulkResult<MediaCollection> = crunchy
            .executor
            .get(endpoint)
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;

        if result.items.is_empty() {
            Err(CrunchyrollError::Input(
                format!("no media could be found for id '{}'", id).into(),
            ))
        } else if result.items.len() >= 2 {
            // if this ever gets fired, crunchyroll hopefully unified episode and movie on the api
            // level (this functions was only implemented so `crunchyroll_rs::parse_url` can work
            // easily)
            Err(CrunchyrollError::Internal(format!("multiple media were found for id '{}'. this shouldn't happen, please report it immediately!", id).into()))
        } else {
            Ok(result.items.into_iter().next().unwrap())
        }
    }
}

macro_rules! impl_try_into_media {
    ($($generic:path = $enum_field:ident)*) => {
        $(
            impl TryInto<Media<$generic>> for MediaCollection {
                type Error = CrunchyrollError;

                fn try_into(self) -> std::result::Result<Media<$generic>, Self::Error> {
                    if let MediaCollection::$enum_field(value) = self {
                        Ok(value)
                    } else {
                        Err(CrunchyrollError::Input(CrunchyrollErrorContext::new(format!("collection is no '{}'", stringify!($generic)).to_string())))
                    }
                }
            }
        )*
    }
}

impl_try_into_media! {
    Series = Series
    Season = Season
    Episode = Episode
    MovieListing = MovieListing
    Movie = Movie
}

macro_rules! impl_from_media {
    ($($generic:path = $enum_field:ident)*) => {
        $(
            impl From<Media<$generic>> for MediaCollection {
                fn from(value: Media<$generic>) -> Self {
                    Self::$enum_field(value)
                }
            }
        )*
    }
}

impl_from_media! {
    Series = Series
    Season = Season
    Episode = Episode
    MovieListing = MovieListing
    Movie = Movie
}

#[derive(Clone, Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct MediaImages {
    pub thumbnail: Option<Vec<Vec<Image>>>,
    pub poster_tall: Option<Vec<Vec<Image>>>,
    pub poster_wide: Option<Vec<Vec<Image>>>,
    pub promo_image: Option<Vec<Vec<Image>>>,
}

/// Base struct which stores all information about series, seasons, episodes, movie listings and
/// movies. The generic this struct takes specifies which media type it actually contains.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault)]
#[serde(bound = "M: Video")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Media<M: Video> {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    #[serde(rename = "__links__")]
    #[serde(default)]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_stream_id_option")]
    pub stream_id: Option<String>,
    #[serde(rename = "playback")]
    pub playback_url: Option<String>,
    pub external_id: String,
    pub channel_id: String,

    pub slug: String,
    pub title: String,
    pub slug_title: String,
    pub promo_title: String,
    pub description: String,
    pub promo_description: String,

    #[serde(alias = "series_metadata")]
    #[serde(alias = "season_metadata")]
    #[serde(alias = "episode_metadata")]
    #[serde(alias = "movie_listing_metadata")]
    #[serde(alias = "movie_metadata")]
    pub metadata: M,

    /// only populated if `Panel` results from search query 'src/search-series'
    pub search_metadata: Option<SearchMetadata>,

    pub images: Option<MediaImages>,

    #[cfg(feature = "__test_strict")]
    #[serde(alias = "type")]
    pub(crate) collection_type: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    pub(crate) new: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) new_content: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) last_public: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    pub(crate) linked_resource_key: crate::StrictValue,
}

#[async_trait::async_trait(?Send)]
impl<M: Video> Request for Media<M> {
    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        #[cfg(feature = "experimental-stabilizations")]
        M::__apply_fixes(executor, self).await;
    }
}

impl<M: Video> PartialEq for Media<M> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<M: Video> Eq for Media<M> {}

impl<M: Video> Media<M> {
    pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Media<M>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/cms/v2/{}/objects/{}",
            crunchy.executor.details.bucket, &id
        );
        let result: BulkResult<Media<M>> = crunchy
            .executor
            .get(endpoint)
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;

        if result.items.is_empty() {
            Err(CrunchyrollError::Input(
                format!("no media could be found for id '{}'", id).into(),
            ))
        } else if result.items.len() >= 2 {
            // if this ever gets fired, crunchyroll hopefully unified episode and movie on the api
            // level (this functions was only implemented so `crunchyroll_rs::parse_url` can work
            // easily)
            Err(CrunchyrollError::Internal(format!("multiple media were found for id '{}'. this shouldn't happen, please report it immediately!", id).into()))
        } else {
            Ok(result.items.into_iter().next().unwrap())
        }
    }

    pub async fn playback(&self) -> Result<PlaybackStream> {
        if let Some(playback_url) = &self.playback_url {
            self.executor.get(playback_url).request().await
        } else {
            Err(CrunchyrollError::Request("no playback id available".into()))
        }
    }
}

impl Media<Series> {
    /// Return the season of the series.
    pub async fn seasons(&self) -> Result<Vec<Media<Season>>> {
        Media::<Season>::from_series_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Season> {
    pub async fn from_series_id(
        crunchy: &Crunchyroll,
        series_id: String,
    ) -> Result<Vec<Media<Season>>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/cms/v2/{}/seasons",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldSeason> = crunchy
            .executor
            .get(endpoint)
            .query(&[("series_id", series_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }

    /// Returns the episodes of the season.
    pub async fn episodes(&self) -> Result<Vec<Media<Episode>>> {
        Media::<Episode>::from_season_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Episode> {
    pub async fn from_season_id(
        crunchy: &Crunchyroll,
        season_id: String,
    ) -> Result<Vec<Media<Episode>>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/cms/v2/{}/episodes",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldEpisode> = crunchy
            .executor
            .get(endpoint)
            .query(&[("season_id", season_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }

    /// Returns the season the episode belongs to.
    pub async fn season(&self) -> Result<Media<Season>> {
        Season::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.metadata.season_id.clone(),
        )
        .await
    }

    /// Returns the series the episode belongs to.
    pub async fn series(&self) -> Result<Media<Series>> {
        Series::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.metadata.series_id.clone(),
        )
        .await
    }
}

impl Media<MovieListing> {
    pub async fn movies(&self) -> Result<Vec<Media<Movie>>> {
        Media::<Movie>::from_movie_listing_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.id.clone(),
        )
        .await
    }
}

impl Media<Movie> {
    pub async fn from_movie_listing_id(
        crunchy: &Crunchyroll,
        movie_listing_id: String,
    ) -> Result<Vec<Media<Movie>>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/cms/v2/{}/movies",
            crunchy.executor.details.bucket
        );
        let result: BulkResult<OldMovie> = crunchy
            .executor
            .get(endpoint)
            .query(&[("movie_listing_id", movie_listing_id)])
            .apply_media_query()
            .apply_locale_query()
            .request()
            .await?;
        Ok(result.items.into_iter().map(|i| i.into()).collect())
    }

    /// Returns the movie listing this movie belongs to.
    pub async fn movie_listing(&self) -> Result<Media<MovieListing>> {
        MovieListing::from_id(
            &Crunchyroll {
                executor: self.executor.clone(),
            },
            self.metadata.movie_listing_id.clone(),
        )
        .await
    }
}

impl Crunchyroll {
    /// Get a media by its id. The `M` generic says what media exactly should be requested. Available
    /// options are [`Series`], [`Season`], [`Episode`], [`MovieListing`] and [`Movie`].
    pub async fn media_from_id<M: Video>(&self, id: impl AsRef<str>) -> Result<Media<M>> {
        Media::from_id(self, id.as_ref().to_string()).await
    }

    pub async fn media_collection_from_id<S: AsRef<str>>(&self, id: S) -> Result<MediaCollection> {
        MediaCollection::from_id(self, id.as_ref().to_string()).await
    }
}

options! {
    SimilarOptions;
    /// Limit of results to return.
    limit(u32, "n") = Some(20),
    /// Specifies the index from which the entries should be returned.
    start(u32, "start") = None
}

macro_rules! impl_from_id {
    ($($media:ident)*) => {
        $(
            impl $media {
                /// Return a [`Media`] implementation of this type by its id.
                pub async fn from_id(crunchy: &Crunchyroll, id: String) -> Result<Media<$media>> {
                    Media::from_id(crunchy, id).await
                }
            }
        )*
    }
}

impl_from_id! {
    Series Season Episode
    MovieListing Movie
}

macro_rules! impl_media_video_collection {
    ($($media_video:ident)*) => {
        $(
            impl Media<$media_video> {
                /// Similar series or movie listing to the current item.
                pub async fn similar(&self, options: SimilarOptions) -> Result<BulkResult<MediaCollection>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v1/{}/similar_to", self.executor.details.account_id.clone()?);
                    self.executor.get(endpoint)
                        .query(&[("guid", &self.id)])
                        .query(&options.into_query())
                        .apply_locale_query()
                        .request()
                        .await
                }
            }
        )*
    }
}

impl_media_video_collection! {
    Series MovieListing
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
struct VideoIntroResult {
    media_id: String,

    #[serde(rename = "startTime")]
    start_time: f64,
    #[serde(rename = "endTime")]
    end_time: f64,
    duration: f64,

    /// Id of the next episode.
    #[serde(rename = "comparedWith")]
    compared_with: String,

    /// It seems that this represents the episode number relative to the season the episode is part
    /// of. But in a weird way. It is, for example, '0003.00' instead of simply 3 if it's the third
    /// episode in a season.
    ordering: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    last_updated: DateTime<Utc>,
}

macro_rules! impl_media_video {
    ($($media_video:ident)*) => {
        $(
            impl Media<$media_video> {
                /// Streams for this episode / movie.
                pub async fn streams(&self) -> Result<VideoStream> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/cms/v2/{}/videos/{}/streams",
                        self.executor.details.bucket, self.stream_id.as_ref().unwrap_or(&self.id)
                    );
                    self.executor.get(endpoint)
                        .apply_media_query()
                        .apply_locale_query()
                        .request()
                        .await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.details.premium || !self.metadata.is_premium_only
                }

                /// Get time _in seconds_ when the episode / movie intro begins and ends.
                pub async fn intro(&self) -> Result<Option<(f64, f64)>> {
                    let endpoint = format!(
                        "https://static.crunchyroll.com/datalab-intro-v2/{}.json",
                        self.id
                    );
                    let raw_result = self.executor.get(endpoint)
                        .request_raw()
                        .await?;
                    let result = String::from_utf8_lossy(raw_result.as_slice());
                    if result.contains("</Error>") {
                        Ok(None)
                    } else {
                        let video_intro_result: VideoIntroResult = serde_json::from_str(&result)?;
                        Ok(Some((video_intro_result.start_time, video_intro_result.end_time)))
                    }
                }
            }
        )*
    }
}

impl_media_video! {
    Episode Movie
}
