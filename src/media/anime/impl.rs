use crate::common::{PaginationBulkResultMeta, Request};
use crate::media::Media;
use crate::{Episode, MediaCollection, Movie, MovieListing, Result, Season, Series};
use chrono::{DateTime, Utc};
use serde::de::{DeserializeOwned, Error, IntoDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Information about the intro of an [`Episode`] or [`Movie`].
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SkipEventsEvent {
    /// Start of the event in seconds.
    pub start: f32,
    /// End of the event in seconds.
    pub end: f32,

    #[cfg(feature = "__test_strict")]
    approver_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    distribution_number: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    title: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    series_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    new: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    r#type: crate::StrictValue,
}

/// Information about skippable events like an intro or credits.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(remote = "Self")]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct SkipEvents {
    #[serde(default)]
    pub recap: Option<SkipEventsEvent>,
    #[serde(default)]
    pub intro: Option<SkipEventsEvent>,
    #[serde(default)]
    pub credits: Option<SkipEventsEvent>,
    #[serde(default)]
    pub preview: Option<SkipEventsEvent>,

    #[cfg(feature = "__test_strict")]
    media_id: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    last_updated: crate::StrictValue,
}

impl<'de> Deserialize<'de> for SkipEvents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut as_map = serde_json::Map::deserialize(deserializer)?;

        let objects_to_check = ["recap", "intro", "credits", "preview"];
        for object in objects_to_check {
            let Some(obj) = as_map.get(object) else {
                continue;
            };
            if obj.as_object().map_or(false, |o| o.is_empty())
                // crunchyroll sometimes has a skip events, but it's lacking start or end times.
                // this is just abstracted away since an event without a start or end doesn't make
                // sense to be wrapped in e.g. an Option
                || obj.get("start").unwrap_or(&Value::Null).is_null()
                || obj.get("end").unwrap_or(&Value::Null).is_null()
            {
                as_map.remove(object);
            }
        }

        SkipEvents::deserialize(
            serde_json::to_value(as_map)
                .map_err(|e| Error::custom(e.to_string()))?
                .into_deserializer(),
        )
        .map_err(|e| Error::custom(e.to_string()))
    }
}

/// Media related to the media which queried this struct.
#[allow(dead_code)]
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct RelatedMedia<T: Request + DeserializeOwned> {
    pub fully_watched: bool,

    pub playhead: u32,

    #[serde(alias = "panel")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_panel")]
    pub media: T,

    /// Only populated if called with [`Episode::next`] or [`Movie::next`].
    pub shortcut: Option<bool>,
}

/// Information about the playhead of an [`Episode`] or [`Movie`].
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct PlayheadInformation {
    pub playhead: u32,

    pub content_id: String,

    pub fully_watched: bool,

    /// Date when the last playhead update was
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub last_modified: DateTime<Utc>,
}

macro_rules! impl_manual_media_deserialize {
    ($($media:ident = $metadata:literal)*) => {
        $(
            impl<'de> Deserialize<'de> for $media {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
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

macro_rules! impl_manual_media_serialize {
    ($($media:ident)*) => {
        $(
            impl serde::Serialize for $media {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    $media::serialize(self, serializer)
                }
            }
        )*
    }
}

impl_manual_media_serialize! {
    Series Season Episode MovieListing Movie
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
                        // if the versions are still `None`, no other versions exist
                        self.versions = re_requested.versions.map_or(Some(vec![]), |v| Some(v))
                    }
                    // remove version id which references to the caller struct
                    if let Some(pos) = self.versions.as_ref().unwrap().iter().position(|v| v.id == self.id) {
                        self.versions.as_mut().unwrap().remove(pos);
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
    #[doc="Get all available other versions (same [`Season`] but different audio locale) for this [`Season`]."]
    Season = "https://www.crunchyroll.com/content/v2/cms/seasons"
    #[doc="Show in which audios this [`Episode`] is also available."]
    #[doc="Get the versions of this [`Episode`] which have the specified audio locale(s). Use [`Episode::available_versions`] to see all supported locale."]
    #[doc="Get all available other versions (same [`Episode`] but different audio locale) for this [`Episode`]."]
    Episode = "https://www.crunchyroll.com/content/v2/cms/episodes"
    #[doc="Show in which audios this [`MovieListing`] is also available."]
    #[doc="Get the versions of this [`MovieListing`] which have the specified audio locale(s). Use [`MovieListing::available_versions`] to see all supported locale."]
    #[doc="Get all available other versions (same [`MovieListing`] but different audio locale) for this [`MovieListing`]"]
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
                            let result: $crate::common::V2BulkResult<MediaCollection, PaginationBulkResultMeta> = options
                                .executor
                                .get(endpoint)
                                .query(&[("n", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            Ok(result.into())
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
                /// Streams for this episode / movie. Crunchyroll has a newer endpoint to request
                /// streams (available via [`Episode::alternative_stream`] /
                /// [`Movie::alternative_stream`]) but it has some kind of rate limiting. Because of
                /// this, this function utilizes the older endpoint which doesn't have a rate limit.
                /// But because this is an older endpoint it could happen that it stops working at
                /// any time.
                pub async fn stream(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_legacy_url(self.executor.clone(), &self.stream_id).await
                }

                /// Streams for this episode / movie. This endpoint triggers a rate limiting if
                /// requested too much over a short time period (the rate limiting may occur as an
                /// error, Crunchyroll doesn't give a hint that a ratelimit is hit). If you need to
                /// query many streams in a short time, consider using [`Episode::stream`] /
                /// [`Movie::stream`].
                /// Note: It seems that Crunchyroll removed the non-drm endpoints for the results of this method, so the
                /// [`crate::media::Stream::dash_streaming_data`] and [`crate::media::Stream::hls_streaming_data`]
                /// functions will always error.
                pub async fn alternative_stream(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_url(self.executor.clone(), "https://www.crunchyroll.com/content/v2/cms/videos", &self.stream_id).await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.premium().await || !self.is_premium_only
                }

                /// Get skippable events like intro or credits.
                pub async fn skip_events(&self) -> Result<SkipEvents> {
                    let endpoint = format!(
                        "https://static.crunchyroll.com/skip-events/production/{}.json",
                        self.id
                    );
                    let raw_result = self.executor.get(endpoint)
                        .request_raw()
                        .await?;
                    let result = String::from_utf8_lossy(raw_result.as_slice());
                    if result.contains("</Error>") {
                        // sometimes crunchyroll just returns a xml error instead of an empty result
                        return Ok(SkipEvents::default())
                    } else {
                        return Ok(serde_json::from_str(&result)?)
                    }
                }

                /// Get time _in seconds_ when the episode / movie intro begins and ends.
                #[deprecated(since = "0.8.4", note = "Use the `skip_events` method instead")]
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
