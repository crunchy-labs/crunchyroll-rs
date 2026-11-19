//! Browse and search related types.

mod search_media {
    use crate::Executor;
    use crate::common::Request;
    use crate::media::{EpisodeRating, RatingStar, RatingStarDetails};
    use crate::{Concert, Episode, MediaCollection, MusicVideo, Series};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_json::Value;
    use std::ops::{Deref, DerefMut};
    use std::sync::Arc;

    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SearchSeriesRating {
        #[serde(alias = "1s")]
        pub one_star: Option<RatingStarDetails>,
        #[serde(alias = "2s")]
        pub two_stars: Option<RatingStarDetails>,
        #[serde(alias = "3s")]
        pub three_stars: Option<RatingStarDetails>,
        #[serde(alias = "4s")]
        pub four_stars: Option<RatingStarDetails>,
        #[serde(alias = "5s")]
        pub five_stars: Option<RatingStarDetails>,

        pub total: u32,
        #[serde(deserialize_with = "crate::internal::serde::deserialize_try_from_string")]
        pub average: f64,

        #[serde(default)]
        #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
        pub rating: Option<RatingStar>,
    }

    /// Like [`Series`], but exclusive for endpoints that search something.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(series))]
    pub struct SearchSeries {
        #[serde(rename = "rating")]
        pub search_rating: SearchSeriesRating,
        #[serde(flatten)]
        series: Series,
    }

    pub type SearchEpisodeRating = EpisodeRating;

    /// Like [`Episode`], but exclusive for endpoints that search something.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(episode))]
    pub struct SearchEpisode {
        #[serde(rename = "rating")]
        pub search_rating: Option<SearchEpisodeRating>,
        #[serde(flatten)]
        episode: Episode,

        #[cfg(feature = "__test_strict")]
        last_public: Option<crate::StrictValue>,
    }

    pub type SearchMovie = SearchSeries;

    #[derive(Clone, Debug, Serialize, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[serde(untagged)]
    pub enum SearchMusicType {
        MusicVideo(MusicVideo),
        Concert(Concert),
    }

    impl Default for SearchMusicType {
        fn default() -> Self {
            Self::MusicVideo(Default::default())
        }
    }

    #[allow(clippy::from_over_into)]
    impl Into<MediaCollection> for SearchMusicType {
        fn into(self) -> MediaCollection {
            match self {
                SearchMusicType::MusicVideo(music_video) => music_video.into(),
                SearchMusicType::Concert(concert) => concert.into(),
            }
        }
    }

    impl<'de> Deserialize<'de> for SearchMusicType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let media = MediaCollection::deserialize(deserializer)
                .map_err(|e| Error::custom(e.to_string()))?;

            match media {
                MediaCollection::MusicVideo(music_video) => Ok(Self::MusicVideo(music_video)),
                MediaCollection::Concert(concert) => Ok(Self::Concert(concert)),
                _ => Err(Error::custom(
                    "expected music video or concert, got other".to_string(),
                )),
            }
        }
    }

    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(music_type))]
    pub struct SearchMusic {
        #[serde(flatten)]
        pub music_type: SearchMusicType,
    }

    macro_rules! expose_wrapped {
        ($($strukt:ident: $field:ident => $field_ty:path)*) => {
            $(
                impl Deref for $strukt {
                    type Target = $field_ty;

                    fn deref(&self) -> &Self::Target {
                        &self.$field
                    }
                }

                impl DerefMut for $strukt {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.$field
                    }
                }

                #[allow(clippy::from_over_into)]
                impl Into<$field_ty> for $strukt {
                    fn into(self) -> $field_ty {
                        self.$field
                    }
                }
            )*
        };
    }

    expose_wrapped! {
        SearchSeries: series => Series
        SearchEpisode: episode => Episode
    }

    /// Like [`MediaCollection`], but exclusive for endpoints that search something.
    #[allow(clippy::large_enum_variant)]
    #[derive(Serialize, Clone, Debug)]
    pub enum SearchMediaCollection {
        Series(SearchSeries),
        Episode(SearchEpisode),
        Movie(SearchMovie),
        Music(SearchMusic),
    }

    #[allow(clippy::from_over_into)]
    impl Into<MediaCollection> for SearchMediaCollection {
        fn into(self) -> MediaCollection {
            match self {
                SearchMediaCollection::Series(series) => series.series.into(),
                SearchMediaCollection::Episode(episode) => episode.episode.into(),
                SearchMediaCollection::Movie(movie) => movie.series.into(),
                SearchMediaCollection::Music(music) => music.music_type.into(),
            }
        }
    }

    impl Default for SearchMediaCollection {
        fn default() -> Self {
            Self::Series(SearchSeries::default())
        }
    }

    impl<'de> Deserialize<'de> for SearchMediaCollection {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let as_map = serde_json::Map::deserialize(deserializer)?;

            let err_conv = |e: serde_json::Error| serde::de::Error::custom(e.to_string());

            let ty = as_map
                .get("type")
                .ok_or_else(|| serde::de::Error::custom("could not find field 'type'"))?
                .as_str()
                .unwrap();

            match ty {
                "series" => Ok(SearchMediaCollection::Series(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                "episode" => Ok(SearchMediaCollection::Episode(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                "movie_listing" => Ok(SearchMediaCollection::Movie(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                "musicVideo" | "musicConcert" => Ok(SearchMediaCollection::Music(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                _ => Err(serde::de::Error::custom(format!(
                    "invalid search type: {ty}"
                ))),
            }
        }
    }

    impl Request for SearchMediaCollection {
        async fn __set_executor(&mut self, executor: Arc<Executor>) {
            match self {
                SearchMediaCollection::Series(series) => {
                    Request::__set_executor(series, executor).await
                }
                SearchMediaCollection::Episode(episode) => {
                    Request::__set_executor(episode, executor).await
                }
                SearchMediaCollection::Movie(movie) => {
                    Request::__set_executor(movie, executor).await
                }
                SearchMediaCollection::Music(music) => {
                    Request::__set_executor(music, executor).await
                }
            }
        }
    }
}

mod browse {
    use crate::categories::Category;
    use crate::common::{Pagination, PaginationBulkResultMeta, V2BulkResult};
    use crate::search::SearchMediaCollection;
    use crate::{Crunchyroll, Locale, Request, Result, enum_values, options};
    use chrono::{DateTime, Utc};
    use futures_util::FutureExt;
    use serde::{Deserialize, Serialize};

    /// Human readable implementation of [`SimulcastSeason`].
    #[derive(Clone, Debug, Default, Deserialize, Serialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SimulcastSeasonLocalization {
        pub title: String,
        pub description: String,
    }

    /// A simulcast season.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SimulcastSeason {
        pub id: String,
        pub localization: SimulcastSeasonLocalization,
    }

    #[allow(dead_code)]
    #[derive(Clone, Debug, Deserialize, smart_default::SmartDefault, Request)]
    #[request(executor(items))]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct BulkSimulcastSeasonResult {
        items: Vec<SimulcastSeason>,
        total: u32,

        #[cfg(feature = "__test_strict")]
        locale: crate::StrictValue,
    }

    enum_values! {
        /// How to sort queried browse results.
        pub enum BrowseSortType {
            Popularity = "popularity"
            NewlyAdded = "newly_added"
            Alphabetical = "alphabetical"
        }
    }

    enum_values! {
        /// Type of browse media.
        pub enum BrowseMediaType {
            Series = "series"
            Movie = "movie_listing"
            Episode = "episode"
        }
    }

    options! {
        /// Options how to browse.
        BrowseOptions;
        /// Specifies the categories of the entries.
        categories(Vec<Category>, "categories") = None,
        /// Specifies whether the entries should be dubbed.
        is_dubbed(bool, "is_dubbed") = None,
        /// Specifies whether the entries should be subbed.
        is_subbed(bool, "is_subbed") = None,
        /// Specifies a particular simulcast season in which the entries should have been aired. Use
        /// [`Crunchyroll::simulcast_seasons`] to get all seasons.
        simulcast_season(String, "seasonal_tag") = None,
        /// Specifies how the entries should be sorted.
        sort(BrowseSortType, "sort_by") = Some(BrowseSortType::NewlyAdded),
        /// Only show entries that were added after this date. Can be used together with
        /// [`BrowseOptions::to_date`] to get a release calendar like response (actually that's the
        /// way Crunchyroll uses to construct the calendar).
        from_date(DateTime::<Utc>, "from_date", |date: DateTime::<Utc>| date.timestamp_millis()) = None,
        /// Only show entries that were added before this date. Can be used together with
        /// [`BrowseOptions::from_date`] to get a release calendar like response (actually that's the
        /// way Crunchyroll uses to construct the calendar).
        to_date(DateTime::<Utc>, "to_date", |date: DateTime::<Utc>| date.timestamp_millis()) = None,
        /// Specifies the media type of the entries.
        media_type(BrowseMediaType, "type") = None
    }

    impl Crunchyroll {
        /// Browses the crunchyroll catalog filtered by the specified options and returns all found
        /// series and movies.
        pub fn browse(&self, options: BrowseOptions) -> Pagination<SearchMediaCollection> {
            Pagination::new(
                |options| {
                    async move {
                        let endpoint = "https://www.crunchyroll.com/content/v2/discover/browse";
                        let result: V2BulkResult<SearchMediaCollection, PaginationBulkResultMeta> =
                            options
                                .executor
                                .clone()
                                .get(endpoint)
                                .query(&options.query)
                                .query(&[("n", options.page_size), ("start", options.start)])
                                .apply_ratings_query()
                                .apply_locale_query()
                                .apply_preferred_audio_locale_query()
                                .request()
                                .await?;
                        Ok(result.into())
                    }
                    .boxed()
                },
                self.executor.clone(),
                Some(options.into_query()),
                None,
            )
        }

        /// Returns all simulcast seasons. The locale specified which language the localization /
        /// human readable name ([`SimulcastSeasonLocalization::title`]) has.
        pub async fn simulcast_seasons(&self, locale: Locale) -> Result<Vec<SimulcastSeason>> {
            let endpoint = "https://www.crunchyroll.com/content/v1/season_list";
            Ok(self
                .executor
                .get(endpoint)
                .query(&[("locale", locale)])
                .request::<BulkSimulcastSeasonResult>()
                .await?
                .items)
        }
    }
}

mod query {
    use crate::Crunchyroll;
    use crate::common::{Pagination, V2BulkResult, V2TypeBulkResult};
    use crate::search::{
        SearchEpisode, SearchMediaCollection, SearchMovie, SearchMusic, SearchSeries,
    };
    use futures_util::FutureExt;

    /// Results when querying Crunchyroll.
    pub struct QueryResults {
        pub top_results: Pagination<SearchMediaCollection>,
        pub series: Pagination<SearchSeries>,
        pub movie: Pagination<SearchMovie>,
        pub episode: Pagination<SearchEpisode>,
        pub music: Pagination<SearchMusic>,
    }

    impl Crunchyroll {
        /// Search the Crunchyroll catalog by a given query / string.
        pub fn query<S: AsRef<str>>(&self, query: S) -> QueryResults {
            QueryResults {
                top_results: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchMediaCollection>> =
                                options
                                    .executor
                                    .get(endpoint)
                                    .query(&[("q", options.extra.get("q").unwrap())])
                                    .query(&[("type", "top_results")])
                                    .query(&[
                                        ("limit", options.page_size),
                                        ("start", options.start),
                                    ])
                                    .apply_ratings_query()
                                    .apply_locale_query()
                                    .request()
                                    .await?;
                            let top_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "top_results")
                                .unwrap_or_default();
                            Ok(top_results.into())
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    None,
                    Some(vec![("q", query.as_ref().to_string())]),
                ),
                series: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchSeries>> = options
                                .executor
                                .get(endpoint)
                                .query(&[("q", options.extra.get("q").unwrap())])
                                .query(&[("type", "series")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_ratings_query()
                                .apply_locale_query()
                                .request()
                                .await?;
                            let series_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "series")
                                .unwrap_or_default();
                            Ok(series_results.into())
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    None,
                    Some(vec![("q", query.as_ref().to_string())]),
                ),
                movie: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchMovie>> = options
                                .executor
                                .get(endpoint)
                                .query(&[("q", options.extra.get("q").unwrap())])
                                .query(&[("type", "movie_listing")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_ratings_query()
                                .apply_locale_query()
                                .request()
                                .await?;
                            let movie_listing_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "movie_listing")
                                .unwrap_or_default();
                            Ok(movie_listing_results.into())
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    None,
                    Some(vec![("q", query.as_ref().to_string())]),
                ),
                episode: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchEpisode>> = options
                                .executor
                                .get(endpoint)
                                .query(&[("q", options.extra.get("q").unwrap())])
                                .query(&[("type", "episode")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_ratings_query()
                                .apply_locale_query()
                                .request()
                                .await?;
                            let episode_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "episode")
                                .unwrap_or_default();
                            Ok(episode_results.into())
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    None,
                    Some(vec![("q", query.as_ref().to_string())]),
                ),
                music: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchMusic>> = options
                                .executor
                                .get(endpoint)
                                .query(&[("q", options.extra.get("q").unwrap())])
                                .query(&[("type", "music")])
                                .query(&[("limit", options.page_size), ("start", options.start)])
                                .apply_locale_query()
                                .request()
                                .await?;
                            let music_results = result
                                .data
                                .into_iter()
                                .find(|r| r.result_type == "music")
                                .unwrap_or_default();
                            Ok(music_results.into())
                        }
                        .boxed()
                    },
                    self.executor.clone(),
                    None,
                    Some(vec![("q", query.as_ref().to_string())]),
                ),
            }
        }
    }
}

pub use browse::*;
pub use query::*;
pub use search_media::*;
