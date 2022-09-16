use crate::utils::{Store, SESSION};
use crunchyroll_rs::common::FromId;
use crunchyroll_rs::rating::{RatingStar, Review, ReviewOptions};
use crunchyroll_rs::{BulkResult, Series};

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = Series::from_id(crunchy, "GY8VEQ95Y".to_string()).await?;
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
