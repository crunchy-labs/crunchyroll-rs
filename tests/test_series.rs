use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll::media::SimilarOptions;
use crunchyroll::{Media, Series};

mod utils;

static SERIES: Store<Media<Series>> = Store::new(|| {
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
