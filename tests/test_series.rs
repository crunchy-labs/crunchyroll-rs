use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::SimilarOptions;
use crunchyroll_rs::Series;

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = Series::from_id(crunchy, "GY8VEQ95Y", None).await?;
        Ok(series)
    })
});

#[tokio::test]
async fn series_from_id() {
    assert_result!(SERIES.get().await)
}

#[tokio::test]
async fn series_seasons() {
    assert_result!(SERIES.get().await.unwrap().seasons(None).await)
}

#[tokio::test]
async fn series_similar() {
    assert_result!(
        SERIES
            .get()
            .await
            .unwrap()
            .similar(SimilarOptions::default())
            .await
    )
}
