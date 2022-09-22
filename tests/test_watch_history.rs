use crate::utils::SESSION;
use crunchyroll::list::WatchHistoryOptions;

mod utils;

#[tokio::test]
async fn watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.watch_history(WatchHistoryOptions::default()).await)
}

#[tokio::test]
async fn clear_watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.clear_watch_history().await)
}
