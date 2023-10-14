use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn categories() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.categories().await)
}

#[tokio::test]
async fn sub_categories() {
    let crunchy = SESSION.get().await.unwrap();
    let categories = crunchy.categories().await.unwrap();
    assert_result!(categories.first().unwrap().sub_categories().await)
}
