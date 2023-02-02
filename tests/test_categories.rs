use crate::utils::SESSION;
use crunchyroll_rs::categories::CategoryInformationOptions;

mod utils;

#[tokio::test]
async fn categories() {
    let crunchy = SESSION.get().await.unwrap();
    let options = CategoryInformationOptions::default();
    assert_result!(crunchy.categories(options.clone()).await)
}
