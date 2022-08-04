use crunchyroll_rs::{Episode, FromId};
use crate::utils::session::get_session;

mod utils;

#[tokio::test]
async fn episode_from_id() {
    let crunchy = &get_session().await.unwrap();

    let series = Episode::from_id(crunchy, "GRDKJZ81Y".to_string())
        .await;

    assert!(series.is_ok(), "{}", series.unwrap_err().to_string())
}
