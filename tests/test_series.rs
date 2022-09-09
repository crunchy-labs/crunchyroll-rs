use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::common::FromId;
use crunchyroll_rs::Series;

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = Series::from_id(crunchy, "GY8VEQ95Y".to_string()).await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn series_from_id() {
    assert_result!(SERIES.get().await)
}

#[tokio::test]
async fn series_seasons() {
    let series = SERIES.get().await.unwrap();
    assert_result!(series.seasons().await)
}
