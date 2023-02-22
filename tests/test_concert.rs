use crate::utils::{Store, SESSION};
use crunchyroll_rs::Concert;

mod utils;

static CONCERT: Store<Concert> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = crunchy.media_from_id("MC27E95748").await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn concert_from_id() {
    assert_result!(CONCERT.get().await)
}

#[tokio::test]
async fn concert_streams() {
    assert_result!(CONCERT.get().await.unwrap().streams().await)
}
