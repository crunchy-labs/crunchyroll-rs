use crunchyroll_rs::{FromId, Series};
use crate::utils::session::get_session;

mod utils;

#[tokio::test]
async fn series_from_id() {
    let crunchy = &get_session().await.unwrap();

    let series = Series::from_id(crunchy, "GY8VEQ95Y".to_string())
        .await;

    assert!(series.is_ok(), "{}", series.unwrap_err().to_string())
}
