use std::collections::HashMap;
use std::sync::Arc;
use serde::Deserialize;
use crate::common::{BulkResult, Collection, Request};
use crate::error::{CrunchyrollError, CrunchyrollErrorContext, Result};
use crate::{Crunchyroll, Executor};

#[derive(Deserialize, Debug)]
#[serde(try_from = "SearchResultsBulkResult")]
pub struct SearchResults {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub top_results: BulkResult<Collection>,
    pub series: BulkResult<Collection>,
    pub movie_listing: BulkResult<Collection>,
    pub episode: BulkResult<Collection>
}

impl Request for SearchResults {
    fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor.clone();

        for collection in self.top_results.items.iter_mut() {
            collection.set_executor(executor.clone());
        }
        for collection in self.series.items.iter_mut() {
            collection.set_executor(executor.clone());
        }
        for collection in self.movie_listing.items.iter_mut() {
            collection.set_executor(executor.clone());
        }
        for collection in self.episode.items.iter_mut() {
            collection.set_executor(executor.clone());
        }
    }
}

impl TryFrom<SearchResultsBulkResult> for SearchResults {
    type Error = CrunchyrollError;

    fn try_from(value: SearchResultsBulkResult) -> std::result::Result<Self, Self::Error> {
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
                _ => return Err(CrunchyrollError::DecodeError(
                    CrunchyrollErrorContext{ message: format!("invalid result type found: '{}'", item.result_type) }
                ))
            };
        }

        Ok(Self {
            executor: Default::default(),
            top_results: top_results.ok_or(CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: "could not find 'top_result' type".into() }
            ))?,
            series: series.ok_or(CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: "could not find 'series' type".into() }
            ))?,
            movie_listing: movie_listing.ok_or(CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: "could not find 'movie_listing' type".into() }
            ))?,
            episode: episode.ok_or(CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: "could not find 'episode' type".into() }
            ))?
        })
    }
}

#[derive(Deserialize)]
struct SearchResultsBulkResult {
    items: [TypeBulkResult<Collection>; 4]
}

#[derive(Deserialize)]
struct TypeBulkResult<T> {
    #[serde(rename = "type")]
    result_type: String,
    items: Vec<T>,
    total: u32
}

impl Crunchyroll {
    pub async fn search(&self, query: String, limit: u32) -> Result<SearchResults> {
        let executor = self.executor.clone();

        let endpoint = "https://beta.crunchyroll.com/content/v1/search";
        let builder = executor.client
            .get(endpoint)
            .query(&HashMap::from([
                ("q", query),
                ("n", limit.to_string()),
                ("type", "".into()),
                ("locale", self.executor.locale.to_string())
            ]));

        executor.request(builder).await
    }
}