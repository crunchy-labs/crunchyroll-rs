use crate::utils::SESSION;
use crate::utils::Store;
use crunchyroll_rs::MovieListing;

mod utils;

static MOVIE_LISTING: Store<MovieListing> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let movie_listing = crunchy.media_from_id("G6JQ8XWXR").await?;
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
