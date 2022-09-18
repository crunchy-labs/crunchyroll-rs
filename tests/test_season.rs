use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::media::Season;
use crunchyroll_rs::Media;

mod utils;

static SEASON: Store<Media<Season>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(Season::from_id(crunchy, "GRZX8KNGY".to_string()).await?)
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
