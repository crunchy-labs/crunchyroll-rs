use crate::utils::{Store, SESSION};
use crunchyroll_rs::list::WatchlistOptions;
use crunchyroll_rs::{Media, Series};

mod utils;

static SERIES: Store<Media<Series>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = Series::from_id(crunchy, "GY8VEQ95Y".to_string()).await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn watchlist() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.watchlist(WatchlistOptions::default()).await)
}

#[tokio::test]
async fn add_to_watchlist() {
    let series = SERIES.get().await.unwrap();
    if series.into_watchlist_entry().await.unwrap().is_none() {
        assert_result!(series.add_to_watchlist().await)
    }
}

#[tokio::test]
async fn remove_from_watchlist() {
    let series = SERIES.get().await.unwrap();
    if let Some(watchlist_entry) = series.into_watchlist_entry().await.unwrap() {
        let result = watchlist_entry.remove().await;
        assert_result!(result)
    }
}

#[tokio::test]
async fn into_watchlist_entry() {
    let series = SERIES.get().await.unwrap();
    assert_result!(series.into_watchlist_entry().await);
}
