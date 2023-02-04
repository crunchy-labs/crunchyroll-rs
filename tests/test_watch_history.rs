use crate::utils::SESSION;
mod utils;
use crunchyroll_rs::list::WatchHistoryEntry;
use futures_util::StreamExt;

#[tokio::test]
async fn watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy
        .watch_history()
        .next()
        .await
        .unwrap_or(Ok(WatchHistoryEntry::default())))
}

#[tokio::test]
async fn clear_watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.clear_watch_history().await)
}
