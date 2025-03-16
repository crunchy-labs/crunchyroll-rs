use crate::utils::SESSION;
use crate::utils::Store;
use crunchyroll_rs::Season;

mod utils;

static SEASON: Store<Season> = Store::new(|| {
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
