use crunchyroll_rs::{FromId, Movie, Playback, Streams};
use crate::utils::SESSION;
use crate::utils::Store;

mod utils;

static MOVIE: Store<Movie> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let movie = Movie::from_id(crunchy, "G25FVGDEK".to_string())
        .await?;
    Ok(movie)
}));

#[tokio::test]
async fn movie_from_id() {
    let movie = MOVIE.get().await;

    assert!(movie.is_ok(), "{}", movie.unwrap_err().to_string())
}

#[tokio::test]
async fn movie_playback() {
    let movie = MOVIE.get().await.unwrap();

    let playback = movie.playback().await;

    assert!(playback.is_ok(), "{}", playback.unwrap_err())
}

#[tokio::test]
async fn movie_streams() {
    let movie = MOVIE.get().await.unwrap();

    let streams = movie.streams().await;

    assert!(streams.is_ok(), "{}", streams.unwrap_err())
}
