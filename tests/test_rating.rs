use crate::utils::{SESSION, Store};
use crunchyroll_rs::Series;

mod utils;

static SERIES: Store<Series> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let series = crunchy.media_from_id("GY8VEQ95Y").await?;
        Ok(series)
    })
});

#[tokio::test]
async fn rating() {
    assert_result!(SERIES.get().await.unwrap().rating().await);
}
