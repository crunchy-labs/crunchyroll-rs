use crunchyroll_rs::search::query::{QueryOptions, QueryType};
use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn by_query() {
    let crunchy = SESSION.get().await.unwrap();

    let default_result = crunchy.query("darling".into(), Default::default()).await;
    assert!(default_result.is_ok(), "{}", default_result.unwrap_err().to_string());

    let zero_limit_result = crunchy.query("test".into(), QueryOptions::default().limit(0)).await;
    assert!(zero_limit_result.is_ok(), "{}", default_result.unwrap_err().to_string());
    let zero_limit_result_unwrapped = zero_limit_result.unwrap();
    assert!(zero_limit_result_unwrapped.top_results.is_none(), "'top_results' is not None");
    assert!(zero_limit_result_unwrapped.series.is_none(), "'series' is not None");
    assert!(zero_limit_result_unwrapped.movie_listing.is_none(), "'movie_listing' is not None");
    assert!(zero_limit_result_unwrapped.episode.is_none(), "'episode' is not None");

    let series_result = crunchy.query("test".into(), QueryOptions::default().result_type(QueryType::Series)).await;
    assert!(series_result.is_ok(), "{}", default_result.unwrap_err().to_string());
    let series_result_unwrapped = series_result.unwrap();
    assert!(series_result_unwrapped.top_results.is_none(), "'top_results' is not None");
    assert!(series_result_unwrapped.series.is_some(), "'series' is not Some");
    assert!(series_result_unwrapped.movie_listing.is_none(), "'movie_listing' is not None");
    assert!(series_result_unwrapped.episode.is_none(), "'episode' is not None");
}