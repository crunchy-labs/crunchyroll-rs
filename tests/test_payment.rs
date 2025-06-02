use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn billing_history() {
    assert_result!(SESSION.get().await.unwrap().billing_history().await)
}
