use crate::utils::SESSION;

mod utils;

#[tokio::test]
async fn tenant_categories() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.tenant_categories(false).await)
}

#[tokio::test]
async fn tenant_categories_with_subcategories() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.tenant_categories(true).await)
}
