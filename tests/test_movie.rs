use crunchyroll_rs::{FromId, Movie};
use crate::utils::session::get_session;

mod utils;

#[tokio::test]
async fn movie_from_id() {
    let crunchy = &get_session().await.unwrap();

    let movie = Movie::from_id(crunchy, "G25FVGDEK".to_string())
        .await;

    assert!(movie.is_ok(), "{}", movie.unwrap_err().to_string())
}
