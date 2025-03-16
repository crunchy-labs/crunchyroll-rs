use crate::utils::{SESSION, Store};
use crunchyroll_rs::MusicVideo;

mod utils;

static MUSIC_VIDEO: Store<MusicVideo> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = crunchy.media_from_id("MV107DAD58").await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn music_video_from_id() {
    assert_result!(MUSIC_VIDEO.get().await)
}

#[tokio::test]
async fn music_video_stream() {
    let stream = MUSIC_VIDEO.get().await.unwrap().stream().await.unwrap();
    stream.invalidate().await.unwrap()
}

#[tokio::test]
async fn music_video_related_anime() {
    assert_result!(MUSIC_VIDEO.get().await.unwrap().related_anime().await)
}

#[tokio::test]
async fn music_video_artist() {
    assert_result!(
        MUSIC_VIDEO.get().await.unwrap().artists.main_artist[0]
            .artist()
            .await
    )
}
