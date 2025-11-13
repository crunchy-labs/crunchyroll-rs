//! Browse and search related types.

mod search_media {
    use crate::common::Request;
    use crate::crunchyroll::Executor;
    use crate::media::{EpisodeRating, RatingStar, RatingStarDetails};
    use crate::{Concert, Episode, MediaCollection, MovieListing, MusicVideo, Series};
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
        pub search_rating: SearchEpisodeRating,
        #[serde(flatten)]
        episode: Episode,
    }

    pub type SearchMovieListingRating = SearchSeriesRating;

    /// Like [`MovieListing`], but exclusive for endpoints that search something.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(movie_listing))]
    pub struct SearchMovieListing {
        #[serde(rename = "rating")]
        pub search_rating: SearchMovieListingRating,
        #[serde(flatten)]
        movie_listing: MovieListing,
    }

    /// Like [`MusicVideo`], but exclusive for endpoints that search something.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(music_video))]
    pub struct SearchMusicVideo {
        #[serde(flatten)]
        music_video: MusicVideo,
    }

    /// Like [`Concert`], but exclusive for endpoints that search something.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[request(executor(concert))]
    pub struct SearchConcert {
        #[serde(flatten)]
        concert: Concert,
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
        SearchMovieListing: movie_listing => MovieListing
        SearchMusicVideo: music_video => MusicVideo
        SearchConcert: concert => Concert
    }

    /// Like [`MediaCollection`], but exclusive for endpoints that search something.
    #[allow(clippy::large_enum_variant)]
    #[derive(Serialize, Clone, Debug)]
    pub enum SearchMediaCollection {
        Series(SearchSeries),
        Episode(SearchEpisode),
        MovieListing(SearchMovieListing),
        MusicVideo(SearchMusicVideo),
        Concert(SearchConcert),
    }

    #[allow(clippy::from_over_into)]
    impl Into<MediaCollection> for SearchMediaCollection {
        fn into(self) -> MediaCollection {
            match self {
                SearchMediaCollection::Series(series) => series.series.into(),
                SearchMediaCollection::Episode(episode) => episode.episode.into(),
                SearchMediaCollection::MovieListing(movie_listing) => {
                    movie_listing.movie_listing.into()
                }
                SearchMediaCollection::MusicVideo(music_video) => music_video.music_video.into(),
                SearchMediaCollection::Concert(concert) => concert.concert.into(),
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
                "movie_listing" => Ok(SearchMediaCollection::MovieListing(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                "musicVideo" => Ok(SearchMediaCollection::MusicVideo(
                    serde_json::from_value(Value::Object(as_map)).map_err(err_conv)?,
                )),
                "musicConcert" => Ok(SearchMediaCollection::Concert(
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
                SearchMediaCollection::MovieListing(movie_listing) => {
                    Request::__set_executor(movie_listing, executor).await
                }
                SearchMediaCollection::MusicVideo(music_video) => {
                    Request::__set_executor(music_video, executor).await
                }
                SearchMediaCollection::Concert(concert) => {
                    Request::__set_executor(concert, executor).await
                }
            }
        }
    }
}

mod browse {
    use crate::categories::Category;
    use crate::common::{Pagination, PaginationBulkResultMeta, V2BulkResult};
    use crate::media::MediaType;
    use crate::search::SearchMediaCollection;
    use crate::{Crunchyroll, Locale, Request, Result, enum_values, options};
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
        /// Specifies the media type of the entries.
        media_type(MediaType, "type") = None
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
        SearchEpisode, SearchMediaCollection, SearchMovieListing, SearchMusicVideo, SearchSeries,
    };
    use futures_util::FutureExt;

    /// Results when querying Crunchyroll.
    pub struct QueryResults {
        pub top_results: Pagination<SearchMediaCollection>,
        pub series: Pagination<SearchSeries>,
        pub movie_listing: Pagination<SearchMovieListing>,
        pub episode: Pagination<SearchEpisode>,
        pub music: Pagination<SearchMusicVideo>,
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
                movie_listing: Pagination::new(
                    |options| {
                        async move {
                            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
                            let result: V2BulkResult<V2TypeBulkResult<SearchMovieListing>> =
                                options
                                    .executor
                                    .get(endpoint)
                                    .query(&[("q", options.extra.get("q").unwrap())])
                                    .query(&[("type", "movie_listing")])
                                    .query(&[
                                        ("limit", options.page_size),
                                        ("start", options.start),
                                    ])
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
                            let result: V2BulkResult<V2TypeBulkResult<SearchMusicVideo>> = options
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
                                .find(|r| r.result_type == "musicVideo")
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
