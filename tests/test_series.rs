use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::SimilarOptions;
use crunchyroll_rs::{Media, Series};

mod utils;

static SERIES: Store<Media<Series>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = Series::from_id(crunchy, "GY8VEQ95Y".to_string()).await?;
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
