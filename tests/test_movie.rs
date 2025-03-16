use crate::utils::SESSION;
use crate::utils::Store;
use crunchyroll_rs::Movie;

mod utils;

static MOVIE: Store<Movie> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie = crunchy.media_from_id("G71F4DJ24").await?;
        Ok(movie)
    })
});

#[tokio::test]
async fn movie_from_id() {
    assert_result!(MOVIE.get().await)
}

#[tokio::test]
async fn movie_stream() {
    let stream = MOVIE.get().await.unwrap().stream().await.unwrap();
    stream.invalidate().await.unwrap()
}
