use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::Movie;

mod utils;

static MOVIE: Store<Movie> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie = Movie::from_id(crunchy, "G25FVGDEK", None).await?;
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
