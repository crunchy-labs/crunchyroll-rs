use crate::utils::{Store, SESSION};
use crunchyroll_rs::feed::{
    HomeFeed, HomeFeedOptions, NewsFeedOptions, RecommendationOptions, UpNextOptions,
};

mod utils;

static HOME_FEED: Store<HomeFeed> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let home_feed_items = crunchy.home_feed(HomeFeedOptions::default()).await?;
        let home_feed = home_feed_items.get(0).unwrap().clone();
        Ok(home_feed)
    })
});

#[tokio::test]
async fn home_feed_by_id() {
    assert_result!(HOME_FEED.get().await);
}

#[tokio::test]
async fn home_feed_type() {
    let home_feed = HOME_FEED.get().await.unwrap();

    assert_result!(home_feed.home_feed_type().await)
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
