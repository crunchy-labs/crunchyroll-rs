use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll::{Episode, Media};

mod utils;

static EPISODE: Store<Media<Episode>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode = crunchy.media_from_id("GRDKJZ81Y").await?;
        Ok(episode)
    })
});

#[tokio::test]
async fn episode_from_id() {
    assert_result!(EPISODE.get().await)
}

#[tokio::test]
async fn episode_playback() {
    let episode = EPISODE.get().await.unwrap();

    assert_result!(episode.playback().await)
}

#[tokio::test]
async fn episode_streams() {
    let episode = EPISODE.get().await.unwrap();

    assert_result!(episode.streams().await)
}
