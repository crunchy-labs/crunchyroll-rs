use crate::common::Request;
use crate::media::Media;
use crate::{Episode, MediaCollection, Movie, MovieListing, Result, Season, Series};
use chrono::{DateTime, Utc};
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};
use serde_json::Map;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub(crate) struct VideoIntroResult {
    pub(crate) media_id: String,

    #[serde(rename = "startTime")]
    pub(crate) start_time: f64,
    #[serde(rename = "endTime")]
    pub(crate) end_time: f64,
    pub(crate) duration: f64,

    /// Id of the next episode.
    #[serde(rename = "comparedWith")]
    pub(crate) compared_with: String,

    /// It seems that this represents the episode number relative to the season the episode is part
    /// of. But in a weird way. It is, for example, '0003.00' instead of simply 3 if it's the third
    /// episode in a season.
    pub(crate) ordering: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub(crate) last_updated: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct RelatedMedia<T: Request + DeserializeOwned> {
    pub fully_watched: bool,

    pub playhead: u32,

    #[serde(alias = "panel")]
    #[serde(deserialize_with = "deserialize_panel")]
    pub media: T,

    #[cfg(feature = "__test_strict")]
    shortcut: Option<crate::StrictValue>,
}

pub(crate) fn deserialize_panel<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let mut as_map = Map::deserialize(deserializer)?;

    if let Some(mut episode_metadata) = as_map.remove("episode_metadata") {
        as_map.append(episode_metadata.as_object_mut().unwrap())
    }

    serde_json::from_value(serde_json::to_value(as_map).map_err(|e| Error::custom(e.to_string()))?)
        .map_err(|e| Error::custom(e.to_string()))
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlayheadInformation {
    playhead: u32,

    content_id: String,

    fully_watched: bool,

    /// Date when the last playhead update was
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    last_modified: DateTime<Utc>,
}

macro_rules! impl_manual_media_deserialize {
    ($($media:ident = $metadata:literal)*) => {
        $(
            impl<'de> serde::Deserialize<'de> for $media {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    use serde::de::{Error, IntoDeserializer};

                    let mut as_map = serde_json::Map::deserialize(deserializer)?;

                    if let Some(mut metadata) = as_map.remove($metadata) {
                        if let Some(object) = metadata.as_object_mut() {
                            as_map.append(object);
                        } else {
                            as_map.insert($metadata.to_string(), metadata);
                        }
                    }

                    $media::deserialize(
                        serde_json::to_value(as_map)
                            .map_err(|e| Error::custom(e.to_string()))?
                            .into_deserializer(),
                    )
                    .map_err(|e| Error::custom(e.to_string()))
                }
            }
        )*
    }
}

impl_manual_media_deserialize! {
    Series = "series_metadata"
    Season = "season_metadata"
    Episode = "episode_metadata"
    MovieListing = "movie_listing_metadata"
    Movie = "movie_metadata"
}

macro_rules! impl_media_request {
    ($($media:ident)*) => {
        $(
            #[async_trait::async_trait]
            impl $crate::common::Request for $media {
                async fn __set_executor(&mut self, executor: std::sync::Arc<$crate::Executor>) {
                    self.executor = executor;

                    self.__apply_fixes().await;
                    #[cfg(feature = "experimental-stabilizations")]
                    self.__apply_experimental_stabilizations().await;
                }
            }
        )*
    }
}

impl_media_request! {
    Series Season Episode MovieListing Movie
}

macro_rules! media_eq {
    ($($media:ident)*) => {
        $(
            impl PartialEq<Self> for $media {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }
        )*
    }
}

media_eq! {
    Series Season Episode MovieListing Movie
}

macro_rules! media_version {
    ($(#[doc=$available_versions_doc:literal] #[doc=$version_doc:literal] #[doc=$versions_doc:literal] $media:ident = $endpoint:literal)*) => {
        $(
            impl $media {
                /// Some requests doesn't populate the `versions` field (e.g. [`Crunchyroll::browse`]).
                /// Every function which interacts with versions calls this function first to assert
                /// that the `versions` field contains valid data. If not, the current media is
                /// re-requested (`from_id` calls are containing the valid `versions` field) and the
                /// `versions` field is updated with the version of the re-requested struct.
                async fn assert_versions(&mut self) -> Result<()> {
                    if self.versions.is_none() {
                        let re_requested = $media::from_id(&$crate::Crunchyroll { executor: self.executor.clone() }, &self.id).await?;
                        self.versions = re_requested.versions
                    }
                    Ok(())
                }

                #[doc=$available_versions_doc]
                pub async fn available_versions(&mut self) -> Result<Vec<$crate::Locale>> {
                    self.assert_versions().await?;
                    Ok(self.versions.as_ref().unwrap().iter().map(|v| v.audio_locale.clone()).collect())
                }

                #[doc=$version_doc]
                pub async fn version(&mut self, audio_locales: Vec<$crate::Locale>) -> Result<Vec<$media>> {
                    self.assert_versions().await?;
                    let version_ids = self.versions.as_ref().unwrap()
                        .iter()
                        .filter_map(|v| if audio_locales.contains(&v.audio_locale) { Some(v.id.clone()) } else { None } )
                        .collect::<Vec<String>>();

                    let mut result = vec![];
                    for id in version_ids {
                        result.push($media::from_id(&$crate::Crunchyroll { executor: self.executor.clone() }, id).await?)
                    }
                    Ok(result)
                }

                #[doc=$versions_doc]
                pub async fn versions(&mut self) -> Result<Vec<$media>> {
                    self.assert_versions().await?;
                    let version_ids = self.versions.as_ref().unwrap().iter().map(|v| v.id.clone()).collect::<Vec<String>>();

                    let mut result = vec![];
                    for id in version_ids {
                        result.push($media::from_id(&$crate::Crunchyroll { executor: self.executor.clone() }, id).await?)
                    }
                    Ok(result)
                }
            }
        )*
    }
}

media_version! {
    #[doc="Show in which audios this [`Season`] is also available."]
    #[doc="Get the versions of this [`Season`] which have the specified audio locale(s). Use [`Season::available_versions`] to see all supported locale."]
    #[doc="Get all available versions (same [`Season`] but different audio locale) for this [`Season`]."]
    Season = "https://www.crunchyroll.com/content/v2/cms/seasons"
    #[doc="Show in which audios this [`Episode`] is also available."]
    #[doc="Get the versions of this [`Episode`] which have the specified audio locale(s). Use [`Episode::available_versions`] to see all supported locale."]
    #[doc="Get all available versions (same [`Episode`] but different audio locale) for this [`Episode`]."]
    Episode = "https://www.crunchyroll.com/content/v2/cms/episodes"
    #[doc="Show in which audios this [`MovieListing`] is also available."]
    #[doc="Get the versions of this [`MovieListing`] which have the specified audio locale(s). Use [`MovieListing::available_versions`] to see all supported locale."]
    #[doc="Get all available versions (same [`MovieListing`] but different audio locale) for this [`MovieListing`]"]
    MovieListing = "https://www.crunchyroll.com/content/v2/cms/movie_listings"
}

macro_rules! impl_media_video_collection {
    ($($media_video:ident)*) => {
        $(
            impl $media_video {
                /// Similar series or movie listing to the current item.
                pub fn similar(&self) -> $crate::common::Pagination<MediaCollection> {
                    use futures_util::FutureExt;

                    $crate::common::Pagination::new(|options| {
                        async move {
                            let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/{}/similar_to/{}", options.executor.details.account_id.clone()?, options.extra.get("id").unwrap());
                            let result: $crate::common::V2BulkResult<MediaCollection> = options
                                .executor
                                .get(endpoint)
                                .query(&[("n", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            Ok((result.data, result.total))
                        }
                        .boxed()
                    }, self.executor.clone(), None, Some(vec![("id", self.id.clone())]))
                }
            }
        )*
    }
}

impl_media_video_collection! {
    Series MovieListing
}

macro_rules! impl_media_video {
    ($($media_video:ident)*) => {
        $(
            impl $media_video {
                /// Streams for this episode / movie.
                pub async fn streams(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_url(self.executor.clone(), "https://www.crunchyroll.com/content/v2/cms/videos", &self.stream_id).await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.details.premium || !self.is_premium_only
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

                /// Return the previous episode / movie. Is [`None`] if the current media is the
                /// first in its season / has no previous media.
                pub async fn previous(&self) -> Result<Option<RelatedMedia<$media_video>>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/previous_episode/{}", &self.id);
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .apply_locale_query()
                        .apply_preferred_audio_locale_query()
                        .request()
                        .await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        let mut previous: $crate::common::V2BulkResult<RelatedMedia<$media_video>> = serde_json::from_value(result)?;
                        Ok(Some(previous.data.remove(0)))
                    }
                }

                /// Return the next episode / movie. Is [`None`] if the current media is the last in
                /// its season / has no further media afterwards.
                pub async fn next(&self) -> Result<Option<RelatedMedia<$media_video>>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/up_next/{}", self.id);
                    let result: serde_json::Value = self.executor.get(endpoint)
                        .apply_locale_query()
                        .apply_preferred_audio_locale_query()
                        .request()
                        .await?;
                    let as_map: serde_json::Map<String, serde_json::Value> = serde_json::from_value(result.clone())?;
                    if as_map.is_empty() {
                        Ok(None)
                    } else {
                        let mut next: $crate::common::V2BulkResult<RelatedMedia<$media_video>> = serde_json::from_value(result)?;
                        Ok(Some(next.data.remove(0)))
                    }
                }

                /// Get playhead information.
                pub async fn playhead(&self) -> Result<Option<PlayheadInformation>> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/{}/playheads", self.executor.details.account_id.clone()?);
                    Ok(self.executor.get(endpoint)
                        .query(&[("content_ids", &self.id)])
                        .apply_locale_query()
                        .request::<$crate::common::V2BulkResult<PlayheadInformation>>()
                        .await?
                        .data
                        .get(0)
                        .cloned())
                }

                /// Set the playhead (current playback position) for this episode / movie. Used unit
                /// is seconds. Setting the playhead also triggers the Crunchyroll Discord
                /// integration so if you update the playhead and have Crunchyroll connected to
                /// Discord, this episode / movie will be shown as your Discord status.
                pub async fn set_playhead(&self, position: u32) -> Result<()> {
                    let endpoint = format!("https://www.crunchyroll.com/content/v2/{}/playheads", self.executor.details.account_id.clone()?);
                    self.executor.post(endpoint)
                        .apply_locale_query()
                        .json(&serde_json::json!({"content_id": &self.id, "playhead": position}))
                        .request::<$crate::EmptyJsonProxy>()
                        .await?;
                    Ok(())
                }
            }
        )*
    }
}

impl_media_video! {
    Episode Movie
}
