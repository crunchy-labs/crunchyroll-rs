#![cfg(feature = "release-calendar")]

use crate::utils::SESSION;
use chrono::{Days, Utc};

mod utils;

#[tokio::test]
async fn release_calendar_today() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(crunchy.release_calendar(Utc::now()).await)
}

#[tokio::test]
async fn release_calendar_last_week() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(crunchy.release_calendar(Utc::now() - Days::new(7)).await)
}
