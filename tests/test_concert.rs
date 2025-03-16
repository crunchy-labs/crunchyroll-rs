use crate::utils::{SESSION, Store};
use crunchyroll_rs::Concert;

mod utils;

static CONCERT: Store<Concert> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let concert = crunchy.media_from_id("MC27E95748").await?;
        Ok(concert)
    })
});

#[tokio::test]
async fn concert_from_id() {
    assert_result!(CONCERT.get().await)
}

#[tokio::test]
async fn concert_stream() {
    let stream = CONCERT.get().await.unwrap().stream().await.unwrap();
    stream.invalidate().await.unwrap()
}
