use crate::utils::SESSION;
use crunchyroll_rs::feed::{
    HomeFeedOptions, NewsFeedOptions, RecommendationOptions, UpNextOptions,
};

mod utils;

#[tokio::test]
async fn home_feed() {
    assert_result!(
        SESSION
            .get()
            .await
            .unwrap()
            .home_feed(HomeFeedOptions::default().limit(100))
            .await
    )
}

#[tokio::test]
async fn news_feed() {
    assert_result!(
        SESSION
            .get()
            .await
            .unwrap()
            .news_feed(NewsFeedOptions::default())
            .await
    )
}

#[tokio::test]
async fn recommendations() {
    assert_result!(
        SESSION
            .get()
            .await
            .unwrap()
            .recommendations(RecommendationOptions::default())
            .await
    )
}

#[tokio::test]
async fn up_next() {
    assert_result!(
        SESSION
            .get()
            .await
            .unwrap()
            .up_next(UpNextOptions::default())
            .await
    )
}
