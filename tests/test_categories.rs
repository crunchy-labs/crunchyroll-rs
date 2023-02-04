use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn categories() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.categories().await)
}
