use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn active_devices() {
    assert_result!(SESSION.get().await.unwrap().active_devices().await)
}
