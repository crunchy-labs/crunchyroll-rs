use crate::utils::SESSION;
use crunchyroll::search::{BrowseOptions, QueryOptions, QueryType};

mod utils;

#[tokio::test]
async fn by_browse() {
    let crunchy = SESSION.get().await.unwrap();

    let default_result = crunchy.browse(Default::default()).await;
    assert_result!(default_result);

    let zero_limit_result = crunchy.browse(BrowseOptions::default().limit(0)).await;
    assert_result!(zero_limit_result);
}

#[tokio::test]
async fn by_query() {
    let crunchy = SESSION.get().await.unwrap();

    let default_result = crunchy.query("darling", Default::default()).await;
    assert_result!(default_result);

    let zero_limit_result = crunchy
        .query("test", QueryOptions::default().limit(0))
        .await;
    assert_result!(zero_limit_result);
    let zero_limit_result_unwrapped = zero_limit_result.unwrap();
    assert!(
        zero_limit_result_unwrapped.top_results.is_none(),
        "'top_results' is not None"
    );
    assert!(
        zero_limit_result_unwrapped.series.is_none(),
        "'series' is not None"
    );
    assert!(
        zero_limit_result_unwrapped.movie_listing.is_none(),
        "'movie_listing' is not None"
    );
    assert!(
        zero_limit_result_unwrapped.episode.is_none(),
        "'episode' is not None"
    );

    let series_result = crunchy
        .query(
            "test",
            QueryOptions::default().result_type(QueryType::Series),
        )
        .await;
    assert_result!(series_result);
    let series_result_unwrapped = series_result.unwrap();
    assert!(
        series_result_unwrapped.top_results.is_none(),
        "'top_results' is not None"
    );
    assert!(
        series_result_unwrapped.series.is_some(),
        "'series' is not Some"
    );
    assert!(
        series_result_unwrapped.movie_listing.is_none(),
        "'movie_listing' is not None"
    );
    assert!(
        series_result_unwrapped.episode.is_none(),
        "'episode' is not None"
    );
}
