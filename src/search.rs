pub mod browse {
    use crate::categories::Category;
    use crate::common::BulkResult;
    use crate::media::MediaType;
    use crate::{enum_values, options, Crunchyroll, MediaCollection, Result};

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
        /// Specifies a particular simulcast season by id in which the entries have been aired.
        simulcast(String, "season_tag") = None,
        /// Specifies how the entries should be sorted.
        sort(BrowseSortType, "sort") = Some(BrowseSortType::NewlyAdded),
        /// Specifies the media type of the entries.
        media_type(MediaType, "type") = None,

        /// Limit of results to return.
        limit(u32, "n") = Some(20),
        /// Specifies the index from which the entries should be returned.
        start(u32, "start") = None
    }

    impl Crunchyroll {
        /// Browses the crunchyroll catalog filtered by the specified options and returns all found
        /// series and movies.
        pub async fn browse(&self, options: BrowseOptions) -> Result<BulkResult<MediaCollection>> {
            let endpoint = "https://beta.crunchyroll.com/content/v1/browse";
            self.executor
                .get(endpoint)
                .query(&options.to_query())
                .apply_locale_query()
                .request()
                .await
        }
    }
}

pub mod query {
    use crate::common::{BulkResult, Request};
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
    use crate::media::{Episode, MovieListing, Series};
    use crate::{enum_values, options, Crunchyroll, Executor, Media, MediaCollection};
    use serde::Deserialize;
    use std::sync::Arc;

    #[derive(Clone, Debug, Default, Deserialize, Request)]
    #[request(executor(top_results, series, movie_listing, episode))]
    #[serde(try_from = "BulkResult<QueryBulkResult>")]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    pub struct QueryResults {
        #[serde(skip)]
        executor: Arc<Executor>,

        pub top_results: Option<BulkResult<MediaCollection>>,
        pub series: Option<BulkResult<Media<Series>>>,
        pub movie_listing: Option<BulkResult<Media<MovieListing>>>,
        pub episode: Option<BulkResult<Media<Episode>>>,
    }

    impl TryFrom<BulkResult<QueryBulkResult>> for QueryResults {
        type Error = CrunchyrollError;

        fn try_from(value: BulkResult<QueryBulkResult>) -> std::result::Result<Self, Self::Error> {
            let mut top_results: Option<BulkResult<MediaCollection>> = None;
            let mut series: Option<BulkResult<Media<Series>>> = None;
            let mut movie_listing: Option<BulkResult<Media<MovieListing>>> = None;
            let mut episode: Option<BulkResult<Media<Episode>>> = None;

            for item in value.items.clone() {
                match item.result_type.as_str() {
                    "top_results" => {
                        top_results = Some(BulkResult {
                            items: item.items,
                            total: item.total,
                        })
                    }
                    "series" => {
                        series = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .map(|i| i.try_into())
                                .collect::<Result<Vec<Media<Series>>>>()?,
                            total: item.total,
                        })
                    }
                    "movie_listing" => {
                        movie_listing = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .map(|i| i.try_into())
                                .collect::<Result<Vec<Media<MovieListing>>>>()?,
                            total: item.total,
                        })
                    }
                    "episode" => {
                        episode = Some(BulkResult {
                            items: item
                                .items
                                .into_iter()
                                .map(|i| i.try_into())
                                .collect::<Result<Vec<Media<Episode>>>>()?,
                            total: item.total,
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
        total: u32,
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
        result_type(QueryType, "type") = None
    }

    impl Crunchyroll {
        /// Search the Crunchyroll catalog by a given query / string.
        pub async fn query(&self, query: String, options: QueryOptions) -> Result<QueryResults> {
            let endpoint = "https://beta.crunchyroll.com/content/v1/search";
            self.executor
                .get(endpoint)
                .query(&options.to_query())
                .query(&[("q", query)])
                .apply_locale_query()
                .request()
                .await
        }
    }
}
