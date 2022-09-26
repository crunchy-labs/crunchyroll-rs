use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll::media::Season;
use crunchyroll::Media;

mod utils;

static SEASON: Store<Media<Season>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(crunchy.media_from_id("GRZX8KNGY").await?)
    })
});

#[tokio::test]
async fn season_from_id() {
    assert_result!(SEASON.get().await)
}

#[tokio::test]
async fn season_episodes() {
    assert_result!(SEASON.get().await.unwrap().episodes().await)
}
