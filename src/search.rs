pub mod query {
    use std::sync::Arc;
    use serde::Deserialize;
    use crate::{Collection, Crunchyroll, enum_values, Executor};
    use crate::common::{BulkResult, Request};
    use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};

    #[derive(Deserialize, Debug)]
    #[serde(try_from = "BulkResult<QueryBulkResult>")]
    pub struct QueryResults {
        #[serde(skip)]
        executor: Arc<Executor>,

        pub top_results: Option<BulkResult<Collection>>,
        pub series: Option<BulkResult<Collection>>,
        pub movie_listing: Option<BulkResult<Collection>>,
        pub episode: Option<BulkResult<Collection>>
    }

    impl Request for QueryResults {
        fn set_executor(&mut self, executor: Arc<Executor>) {
            self.executor = executor.clone();

            if let Some(top_results) = &mut self.top_results {
                for collection in top_results.items.iter_mut() {
                    collection.set_executor(executor.clone());
                }
            }
            if let Some(series) = &mut self.series {
                for collection in series.items.iter_mut() {
                    collection.set_executor(executor.clone());
                }
            }
            if let Some(movie_listing) = &mut self.movie_listing {
                for collection in movie_listing.items.iter_mut() {
                    collection.set_executor(executor.clone());
                }
            }
            if let Some(episode) = &mut self.episode {
                for collection in episode.items.iter_mut() {
                    collection.set_executor(executor.clone());
                }
            }
        }
    }

    impl TryFrom<BulkResult<QueryBulkResult>> for QueryResults {
        type Error = CrunchyrollError;

        fn try_from(value: BulkResult<QueryBulkResult>) -> std::result::Result<Self, Self::Error> {
            let mut top_results: Option<BulkResult<Collection>> = None;
            let mut series: Option<BulkResult<Collection>> = None;
            let mut movie_listing: Option<BulkResult<Collection>> = None;
            let mut episode: Option<BulkResult<Collection>> = None;

            for item in value.items {
                let result = BulkResult{ items: item.items, total: item.total };
                match item.result_type.as_str() {
                    "top_results" => top_results = Some(result),
                    "series" => series = Some(result),
                    "movie_listing" => movie_listing = Some(result),
                    "episode" => episode = Some(result),
                    _ => return Err(CrunchyrollError::Decode(
                        CrunchyrollErrorContext{ message: format!("invalid result type found: '{}'", item.result_type) }
                    ))
                };
            }

            Ok(Self {
                executor: Default::default(),
                top_results,
                series,
                movie_listing,
                episode
            })
        }
    }

    #[derive(Deserialize, Default)]
    struct QueryBulkResult {
        #[serde(rename = "type")]
        result_type: String,
        items: Vec<Collection>,
        total: u32
    }

    enum_values!{
        QueryType,
        #[derive(Debug)],
        Series = "series",
        MovieListing = "movie_listing",
        Episode = "episode"
    }

    #[derive(derive_setters::Setters, smart_default::SmartDefault)]
    pub struct QueryOptions {
        #[default = 20]
        pub limit: u32,
        pub result_type: Option<QueryType>
    }

    impl Crunchyroll {
        pub async fn query(&self, query: String, options: QueryOptions) -> Result<QueryResults> {
            let executor = self.executor.clone();

            let endpoint = "https://beta.crunchyroll.com/content/v1/search";
            let builder = executor.client
                .get(endpoint)
                .query(&[
                    ("q", query),
                    ("n", options.limit.to_string()),
                    ("type", options.result_type.map_or_else(|| "".to_string(), |f| format!("{:?}", f)).to_lowercase()),
                    ("locale", self.executor.locale.to_string())
                ]);

            executor.request(builder).await
        }
    }
}
