mod browse {
    use crate::categories::Category;
    use crate::common::V2BulkResult;
    use crate::media::MediaType;
    use crate::{enum_values, options, Crunchyroll, Locale, MediaCollection, Request, Result};
    use serde::Deserialize;

    /// Human readable implementation of [`SimulcastSeason`].
    #[derive(Clone, Debug, Default, Deserialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct SimulcastSeasonLocalization {
        pub title: String,
        pub description: String,
    }

    /// A simulcast season.
    #[derive(Clone, Debug, Default, Deserialize, Request)]
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
        pub enum BrowseSortType {
            Popularity = "popularity"
            NewlyAdded = "newly_added"
            Alphabetical = "alphabetical"
        }
    }

    options! {
        BrowseOptions;
        /// Specifies the categories of the entries.
        categories(Vec<Category>, "categories") = None,
        /// Specifies whether the entries should be dubbed.
        is_dubbed(bool, "is_dubbed") = None,
        /// Specifies whether the entries should be subbed.
        is_subbed(bool, "is_subbed") = None,
        /// Specifies a particular simulcast season in which the entries should have been aired. Use
        /// [`Crunchyroll::simulcast_seasons`] to get all seasons.
        simulcast_season(String, "season_tag") = None,
        /// Specifies how the entries should be sorted.
        sort(BrowseSortType, "sort") = Some(BrowseSortType::NewlyAdded),
        /// Specifies the media type of the entries.
        media_type(MediaType, "type") = None,
        /// Preferred audio language.
        preferred_audio_language(Locale, "preferred_audio_language") = None,

        /// Limit of results to return.
        limit(u32, "n") = Some(20),
        /// Specifies the index from which the entries should be returned.
        start(u32, "start") = None
    }

    impl Crunchyroll {
        /// Browses the crunchyroll catalog filtered by the specified options and returns all found
        /// series and movies.
        pub async fn browse(
            &self,
            options: BrowseOptions,
        ) -> Result<V2BulkResult<MediaCollection>> {
            let endpoint = "https://www.crunchyroll.com/content/v2/discover/browse";
            self.executor
                .get(endpoint)
                .query(&options.into_query())
                .apply_locale_query()
                .request()
                .await
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
    use crate::common::{BulkResult, Request, V2BulkResult};
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
    use crate::media::{Episode, MovieListing, Series};
    use crate::{enum_values, options, Crunchyroll, Executor, Locale, MediaCollection};
    use serde::Deserialize;
    use std::sync::Arc;

    /// Results when querying Crunchyroll. Results depending on the input which was given via
    /// [`QueryOptions::result_type`]. If not specified, every field is populated, if one specific
    /// type, for example [`QueryType::Series`], were provided, only [`QueryResults::series`] will
    /// be populated.
    #[derive(Clone, Debug, Default, Deserialize, Request)]
    #[request(executor(top_results, series, movie_listing, episode))]
    #[serde(try_from = "V2BulkResult<QueryBulkResult>")]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    pub struct QueryResults {
        #[serde(skip)]
        executor: Arc<Executor>,

        pub top_results: Option<BulkResult<MediaCollection>>,
        pub series: Option<BulkResult<Series>>,
        pub movie_listing: Option<BulkResult<MovieListing>>,
        pub episode: Option<BulkResult<Episode>>,
    }

    impl TryFrom<V2BulkResult<QueryBulkResult>> for QueryResults {
        type Error = CrunchyrollError;

        fn try_from(
            value: V2BulkResult<QueryBulkResult>,
        ) -> std::result::Result<Self, Self::Error> {
            let mut top_results: Option<BulkResult<MediaCollection>> = None;
            let mut series: Option<BulkResult<Series>> = None;
            let mut movie_listing: Option<BulkResult<MovieListing>> = None;
            let mut episode: Option<BulkResult<Episode>> = None;

            for item in value.data.clone() {
                match item.result_type.as_str() {
                    "top_results" => {
                        top_results = Some(BulkResult {
                            items: item.items,
                            total: item.count,
                        })
                    }
                    "series" => {
                        series = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .filter_map(|i| {
                                    if let MediaCollection::Series(s) = i {
                                        Some(s)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<Series>>(),
                            total: item.count,
                        })
                    }
                    "movie_listing" => {
                        movie_listing = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .filter_map(|i| {
                                    if let MediaCollection::MovieListing(ml) = i {
                                        Some(ml)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<MovieListing>>(),
                            total: item.count,
                        })
                    }
                    "episode" => {
                        episode = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .filter_map(|i| {
                                    if let MediaCollection::Episode(e) = i {
                                        Some(e)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<Episode>>(),
                            total: item.count,
                        })
                    }
                    _ => {
                        return Err(CrunchyrollError::Internal(
                            CrunchyrollErrorContext::new(format!(
                                "invalid result type found: '{}'",
                                item.result_type
                            ))
                            .with_value(format!("{:?}", value).as_bytes()),
                        ))
                    }
                };
            }

            Ok(Self {
                executor: Default::default(),
                top_results,
                series,
                movie_listing,
                episode,
            })
        }
    }

    #[derive(Clone, Debug, Default, Deserialize, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct QueryBulkResult {
        #[serde(rename = "type")]
        result_type: String,
        items: Vec<MediaCollection>,
        count: u32,
    }

    enum_values! {
        pub enum QueryType {
            Series = "series"
            MovieListing = "movie_listing"
            Episode = "episode"
        }
    }

    options! {
        QueryOptions;
        /// Limit of results to return.
        limit(u32, "n") = Some(20),
        /// "Type of result to return.
        result_type(QueryType, "type") = None,
        /// Preferred audio language.
        preferred_audio_language(Locale, "preferred_audio_language") = None
    }

    impl Crunchyroll {
        /// Search the Crunchyroll catalog by a given query / string.
        pub async fn query<S: AsRef<str>>(
            &self,
            query: S,
            options: QueryOptions,
        ) -> Result<QueryResults> {
            let endpoint = "https://www.crunchyroll.com/content/v2/discover/search";
            self.executor
                .get(endpoint)
                .query(&options.into_query())
                .query(&[("q", query.as_ref())])
                .apply_locale_query()
                .request()
                .await
        }
    }
}

pub use browse::*;
pub use query::*;
