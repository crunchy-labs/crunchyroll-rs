use crate::utils::SESSION;
use crate::utils::Store;
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
async fn episode_stream() {
    let episode = START_EPISODE.get().await.unwrap();

    let stream = episode.stream().await.unwrap();
    stream.invalidate().await.unwrap()
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
async fn episode_skip_events() {
    let episode = START_EPISODE.get().await.unwrap();
    episode.skip_events().await.unwrap();
}
