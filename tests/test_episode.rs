use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::Episode;

mod utils;

static START_EPISODE: Store<Episode> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode = crunchy.media_from_id("GRDKJZ81Y").await?;
        Ok(episode)
    })
});
static END_EPISODE: Store<Episode> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let episode = crunchy.media_from_id("G6QW40DE6").await?;
        Ok(episode)
    })
});

#[tokio::test]
async fn episode_from_id() {
    assert_result!(START_EPISODE.get().await)
}

#[tokio::test]
async fn episode_streams() {
    let episode = START_EPISODE.get().await.unwrap();

    assert_result!(episode.streams().await)
}

#[tokio::test]
async fn episode_get_playhead() {
    let episode = START_EPISODE.get().await.unwrap();

    assert_result!(episode.playhead().await)
}

#[tokio::test]
async fn episode_set_playhead() {
    let episode = START_EPISODE.get().await.unwrap();

    assert_result!(episode.set_playhead(69).await)
}

#[tokio::test]
async fn episode_some_previous() {
    let episode = END_EPISODE.get().await.unwrap();

    assert_result!(episode.previous().await)
}

#[tokio::test]
async fn episode_none_previous() {
    let episode = START_EPISODE.get().await.unwrap();

    assert_result!(episode.previous().await)
}

#[tokio::test]
async fn episode_some_next() {
    let episode = START_EPISODE.get().await.unwrap();

    assert_result!(episode.next().await)
}

#[tokio::test]
async fn episode_none_next() {
    let episode = END_EPISODE.get().await.unwrap();

    assert_result!(episode.next().await)
}

#[tokio::test]
async fn episode_versions() {
    let mut episode = END_EPISODE.get().await.unwrap().clone();

    assert_result!(episode.versions().await)
}
