use crunchyroll_rs::{FromId, Series};
use crate::utils::SESSION;
use crate::utils::Store;

mod utils;

static SERIES: Store<Series> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let movie_listing = Series::from_id(crunchy, "GY8VEQ95Y".to_string())
        .await?;
    Ok(movie_listing)
}));

#[tokio::test]
async fn series_from_id() {
    let series = SERIES.get().await;

    assert!(series.is_ok(), "{}", series.unwrap_err().to_string())
}
