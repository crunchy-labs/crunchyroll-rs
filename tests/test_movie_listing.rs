use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::{FromId, MovieListing};

mod utils;

static MOVIE_LISTING: Store<MovieListing> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = MovieListing::from_id(crunchy, "G6MG10746".to_string()).await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn movie_listing_from_id() {
    assert_result!(MOVIE_LISTING.get().await)
}
