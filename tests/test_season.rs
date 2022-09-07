use crate::utils::Store;
use crate::utils::SESSION;
use crunchyroll_rs::{FromId, Season};

mod utils;

static SEASON: Store<Season> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(Season::from_id(crunchy, "GRZX8KNGY".to_string()).await?)
    })
});

#[tokio::test]
async fn series_from_id() {
    assert_result!(SEASON.get().await)
}
