use crate::utils::{Store, SESSION};
use crunchyroll_rs::feed::HomeFeed;
use futures_util::StreamExt;

mod utils;

static HOME_FEED: Store<HomeFeed> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let mut home_feed_items = crunchy.home_feed();
        let home_feed = home_feed_items.next().await.unwrap()?;
        Ok(home_feed)
    })
});

#[tokio::test]
async fn home_feed_by_id() {
    assert_result!(HOME_FEED.get().await);
}

#[tokio::test]
async fn news_feed() {
    assert_result!(SESSION
        .get()
        .await
        .unwrap()
        .news_feed()
        .latest_news
        .next()
        .await
        .unwrap())
}

#[tokio::test]
async fn recommendations() {
    assert_result!(SESSION
        .get()
        .await
        .unwrap()
        .recommendations()
        .next()
        .await
        .unwrap())
}
