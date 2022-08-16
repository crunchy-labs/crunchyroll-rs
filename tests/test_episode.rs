use crunchyroll_rs::{Episode, FromId, Playback, Streams};
use crate::utils::SESSION;
use crate::utils::Store;

mod utils;

static EPISODE: Store<Episode> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let episode = Episode::from_id(crunchy, "GRDKJZ81Y".to_string())
        .await?;
    Ok(episode)
}));

#[tokio::test]
async fn episode_from_id() {
    let episode = EPISODE.get().await;

    assert!(episode.is_ok(), "{}", episode.unwrap_err())
}

#[tokio::test]
async fn episode_playback() {
    let episode = EPISODE.get().await.unwrap();

    let playback = episode.playback().await;

    assert!(playback.is_ok(), "{}", playback.unwrap_err())
}

#[tokio::test]
async fn episode_streams() {
    let episode = EPISODE.get().await.unwrap();

    let streams = episode.streams().await;

    assert!(streams.is_ok(), "{}", streams.unwrap_err())
}
