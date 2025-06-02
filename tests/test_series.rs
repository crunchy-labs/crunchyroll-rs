use crate::utils::SESSION;
use crate::utils::Store;
use crunchyroll_rs::Series;
use futures_util::StreamExt;

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = crunchy.media_from_id("GY8VEQ95Y").await?;
        Ok(series)
    })
});

#[tokio::test]
async fn series_from_id() {
    assert_result!(SERIES.get().await)
}

#[tokio::test]
async fn series_seasons() {
    assert_result!(SERIES.get().await.unwrap().seasons().await)
}

#[tokio::test]
async fn series_copyright() {
    assert_result!(SERIES.get().await.unwrap().copyright().await)
}

#[tokio::test]
async fn series_featured_music() {
    assert_result!(SERIES.get().await.unwrap().featured_music().await)
}

#[tokio::test]
async fn series_similar() {
    assert_result!(SERIES.get().await.unwrap().similar().next().await.unwrap())
}
