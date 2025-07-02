use crate::common::Request;
use crate::crunchyroll::Executor;
use crate::error::Error;
use crate::media::Media;
use crate::{
    Concert, Crunchyroll, Episode, Movie, MovieListing, MusicVideo, Result, Season, Series,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Collection of all media types. Useful in situations where [`Media`] can contain more than one
/// specific media.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum MediaCollection {
    Series(Series),
    Season(Season),
    Episode(Episode),
    MovieListing(MovieListing),
    Movie(Movie),
    MusicVideo(MusicVideo),
    Concert(Concert),
}

impl MediaCollection {
    pub async fn from_id<S: AsRef<str>>(
        crunchyroll: &Crunchyroll,
        id: S,
    ) -> Result<MediaCollection> {
        if let Ok(episode) = Episode::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::Episode(episode))
        } else if let Ok(movie) = Movie::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::Movie(movie))
        } else if let Ok(series) = Series::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::Series(series))
        } else if let Ok(season) = Season::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::Season(season))
        } else if let Ok(movie_listing) = MovieListing::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::MovieListing(movie_listing))
        } else if let Ok(concert) = Concert::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::Concert(concert))
        } else if let Ok(music_video) = MusicVideo::from_id(crunchyroll, id.as_ref()).await {
            Ok(MediaCollection::MusicVideo(music_video))
        } else {
            Err(Error::Input {
                message: format!("failed to find valid media with id '{}'", id.as_ref()),
            })
        }
    }
}

impl Default for MediaCollection {
    fn default() -> Self {
        Self::Series(Series::default())
    }
}

impl<'de> Deserialize<'de> for MediaCollection {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_map = serde_json::Map::deserialize(deserializer)?;

        let err_conv = |e: serde_json::Error| serde::de::Error::custom(e.to_string());

        if as_map.contains_key("series_metadata") || as_map.contains_key("series_launch_year") {
            Ok(MediaCollection::Series(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("season_metadata")
            || as_map.contains_key("number_of_episodes")
        {
            Ok(MediaCollection::Season(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("episode_metadata") || as_map.contains_key("sequence_number")
        {
            Ok(MediaCollection::Episode(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_listing_metadata")
            || as_map.contains_key("movie_release_year")
        {
            Ok(MediaCollection::MovieListing(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("movie_metadata")
            || as_map.contains_key("movie_listing_title")
        {
            Ok(MediaCollection::Movie(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else if as_map.contains_key("animeIds") {
            Ok(MediaCollection::MusicVideo(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        // music video contains this field too so music video must be checked before this condition
        } else if as_map.contains_key("availability") {
            Ok(MediaCollection::Concert(
                serde_json::from_value(Value::from(as_map)).map_err(err_conv)?,
            ))
        } else {
            Err(serde::de::Error::custom(
                "could not deserialize into media collection".to_string(),
            ))
        }
    }
}

impl Request for MediaCollection {
    async fn __set_executor(&mut self, executor: Arc<Executor>) {
        match self {
            MediaCollection::Series(series) => Request::__set_executor(series, executor).await,
            MediaCollection::Season(season) => Request::__set_executor(season, executor).await,
            MediaCollection::Episode(episode) => Request::__set_executor(episode, executor).await,
            MediaCollection::MovieListing(movie_listing) => {
                Request::__set_executor(movie_listing, executor).await
            }
            MediaCollection::Movie(movie) => Request::__set_executor(movie, executor).await,
            MediaCollection::MusicVideo(music_video) => {
                Request::__set_executor(music_video, executor).await
            }
            MediaCollection::Concert(concert) => Request::__set_executor(concert, executor).await,
        }
    }
}
