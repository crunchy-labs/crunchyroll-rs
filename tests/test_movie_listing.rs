use crunchyroll_rs::{FromId, MovieListing};
use crate::utils::session::get_session;

mod utils;

#[tokio::test]
async fn movie_listing_from_id() {
    let crunchy = &get_session().await.unwrap();

    let movie_listing = MovieListing::from_id(crunchy, "G6MG10746".to_string())
        .await;

    assert!(movie_listing.is_ok(), "{}", movie_listing.unwrap_err().to_string())
}
