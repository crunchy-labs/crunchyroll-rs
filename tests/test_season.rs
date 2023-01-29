use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::Season;

mod utils;

static SEASON: Store<Season> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(Season::from_id(crunchy, "GRZX8KNGY", None).await?)
    })
});

#[tokio::test]
async fn season_from_id() {
    assert_result!(SEASON.get().await)
}

#[tokio::test]
async fn season_episodes() {
    assert_result!(SEASON.get().await.unwrap().episodes(None).await)
}
