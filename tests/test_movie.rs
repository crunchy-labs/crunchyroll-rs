use crunchyroll_rs::{FromId, Movie};
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
