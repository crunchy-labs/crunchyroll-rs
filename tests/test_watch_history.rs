use crate::utils::SESSION;
mod utils;
use futures_util::StreamExt;

#[tokio::test]
async fn watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.watch_history().next().await.unwrap())
}

#[tokio::test]
async fn clear_watch_history() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.clear_watch_history().await)
}
