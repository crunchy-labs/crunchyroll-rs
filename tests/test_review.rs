use crate::utils::{Store, SESSION};
use crunchyroll_rs::rating::{RatingStar, Review, ReviewOptions};
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
static REVIEW: Store<Review> = Store::new(|| {
    Box::pin(async {
        let series = SERIES.get().await?;
        let mut review = series.reviews(ReviewOptions::default())?;
        Ok(review.next().await.unwrap()?)
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
    assert_result!(REVIEW.get().await)
}
