use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll::{Media, Movie};

mod utils;

static MOVIE: Store<Media<Movie>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie = crunchy.movie_from_id("G25FVGDEK".into()).await?;
        Ok(movie)
    })
});

#[tokio::test]
async fn movie_from_id() {
    assert_result!(MOVIE.get().await)
}

#[tokio::test]
async fn movie_playback() {
    assert_result!(MOVIE.get().await.unwrap().playback().await)
}

#[tokio::test]
async fn movie_streams() {
    assert_result!(MOVIE.get().await.unwrap().streams().await)
}
