use crunchyroll_rs::{FromId, MovieListing};
use crate::utils::SESSION;
use crate::utils::Store;

mod utils;

static MOVIE_LISTING: Store<MovieListing> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let movie_listing = MovieListing::from_id(crunchy, "G6MG10746".to_string())
        .await?;
    Ok(movie_listing)
}));

#[tokio::test]
async fn movie_listing_from_id() {
    let movie_listing = MOVIE_LISTING.get().await;

    assert!(movie_listing.is_ok(), "{}", movie_listing.unwrap_err().to_string())
}
