use crate::common::PaginationBulkResultMeta;
use crate::media::Media;
use crate::media::SkipEvents;
use crate::media::anime::shared::{PlayheadInformation, Rating, RatingStar, RelatedMedia};
use crate::search::SearchMediaCollection;
use crate::{Episode, Movie, MovieListing, Result, Season, Series};
use serde::de::{Error, IntoDeserializer};
use serde::{Deserialize, Deserializer};

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
            impl $crate::common::Request for $media {
                async fn __set_executor(&mut self, executor: std::sync::Arc<$crate::Executor>) {
                    crate::media::Media::__set_executor(self, executor).await;
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

macro_rules! impl_media_video_collection {
    ($($media_video:ident = $endpoint:literal)*) => {
        $(
            impl $media_video {
                /// Similar series or movie listing to the current item.
                pub fn similar(&self) -> $crate::common::Pagination<SearchMediaCollection> {
                    use futures_util::FutureExt;

                    $crate::common::Pagination::new(|options| {
                        async move {
                            let endpoint = format!("https://www.crunchyroll.com/content/v2/discover/{}/similar_to/{}", options.executor.details.account_id.clone()?, options.extra.get("id").unwrap());
                            let result: $crate::common::V2BulkResult<SearchMediaCollection, PaginationBulkResultMeta> = options
                                .executor
                                .get(endpoint)
                                .query(&[("n", options.page_size), ("start", options.start)])
                                .apply_ratings_query()
                                .apply_locale_query()
                                .request()
                                .await?;
                            Ok(result.into())
                        }
                        .boxed()
                    }, self.executor.clone(), None, Some(vec![("id", self.id.clone())]))
                }

                pub async fn rating(&self) -> Result<Rating> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, $endpoint, self.id
                    );
                    self.executor.get(endpoint).request().await
                }

                pub async fn rate(&self, stars: RatingStar) -> Result<Rating> {
                    let endpoint = format!(
                        "https://www.crunchyroll.com/content-reviews/v2/user/{}/rating/{}/{}",
                        self.executor.details.account_id.clone()?, $endpoint, self.id
                    );
                    self.executor.put(endpoint)
                        .json(&serde_json::json!({"rating": stars}))
                        .request()
                        .await
                }
            }
        )*
    }
}

impl_media_video_collection! {
    Series = "series"
    MovieListing = "movie_listing"
}

macro_rules! impl_media_video {
    ($($media_video:ident)*) => {
        $(
            impl $media_video {
                /// Streams for this episode / movie.
                /// All streams are drm encrypted, decryption is not handled in this crate, so you
                /// must do this yourself.
                pub async fn stream(&self) -> Result<$crate::media::Stream> {
                    $crate::media::Stream::from_id(&$crate::Crunchyroll { executor: self.executor.clone() }, &self.id, &self.executor.details.stream_platform).await
                }

                /// Check if the episode / movie can be watched.
                pub async fn available(&self) -> bool {
                    self.executor.premium().await || !self.is_premium_only
                }

                /// Get skippable events like intro or credits.
                pub async fn skip_events(&self) -> Result<Option<SkipEvents>> {
                    let endpoint = format!(
                        "https://static.crunchyroll.com/skip-events/production/{}.json",
                        self.id
                    );
                    self.executor.get(&endpoint).request_static().await
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
