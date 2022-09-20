use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::{Media, MovieListing};

mod utils;

static MOVIE_LISTING: Store<Media<MovieListing>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = crunchy.movie_listing_from_id("G6MG10746".into()).await?;
        Ok(movie_listing)
    })
});

#[tokio::test]
async fn movie_listing_from_id() {
    assert_result!(MOVIE_LISTING.get().await)
}

#[tokio::test]
async fn movies() {
    assert_result!(MOVIE_LISTING.get().await.unwrap().movies().await)
}
