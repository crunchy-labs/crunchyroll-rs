use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn by_query() {
    let crunchy = SESSION.get().await.unwrap();

    let result = crunchy.search("darling".into(), 5).await;

    assert!(result.is_ok(), "{}", result.unwrap_err().to_string())
}