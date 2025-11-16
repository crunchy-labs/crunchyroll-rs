use crate::utils::SESSION;
use crunchyroll_rs::{
    Locale,
    search::{BrowseMediaType, BrowseOptions, BrowseSortType},
};

use futures_util::StreamExt;

mod utils;

#[tokio::test]
async fn by_browse() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(crunchy.browse(Default::default()).next().await.unwrap());
}

#[tokio::test]
async fn by_browse_latest_episodes() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(
        crunchy
            .browse(
                BrowseOptions::default()
                    .sort(BrowseSortType::NewlyAdded)
                    .media_type(BrowseMediaType::Episode),
            )
            .next()
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn by_query() {
    let crunchy = SESSION.get().await.unwrap();

    let mut default_result = crunchy.query("darling");
    assert_result!(default_result.top_results.next().await.unwrap());
    assert_result!(default_result.series.next().await.unwrap());
    // movie listings might or might be not present across different countries for this search term
    // so this is a workaround to keep the test passing
    if let Some(result) = default_result.movie_listing.next().await {
        assert_result!(result)
    }
    assert_result!(default_result.episode.next().await.unwrap())
}

#[tokio::test]
async fn simulcast_seasons() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(crunchy.simulcast_seasons(Locale::en_US).await)
}
