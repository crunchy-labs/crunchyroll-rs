use crate::utils::{Store, SESSION};
use crunchyroll_rs::common::BulkResult;
use crunchyroll_rs::rating::{RatingStar, Review, ReviewOptions};
use crunchyroll_rs::Series;

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = crunchy.media_from_id("GY8VEQ95Y").await?;
        Ok(series)
    })
});
static REVIEWS: Store<BulkResult<Review>> = Store::new(|| {
    Box::pin(async {
        let series = SERIES.get().await?;
        let review = series.reviews(ReviewOptions::default()).await?;
        Ok(review)
    })
});

#[tokio::test]
async fn rating() {
    assert_result!(SERIES.get().await.unwrap().rating().await);
}

#[tokio::test]
async fn rate() {
    assert_result!(
        SERIES
            .get()
            .await
            .unwrap()
            .rate(RatingStar::FiveStars)
            .await
    );
}

#[tokio::test]
async fn reviews() {
    assert_result!(REVIEWS.get().await)
}
