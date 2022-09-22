use crate::utils::SESSION;
use crunchyroll::categories::TenantCategoryOptions;

mod utils;

#[tokio::test]
async fn tenant_categories() {
    let crunchy = SESSION.get().await.unwrap();
    let options = TenantCategoryOptions::default().include_subcategories(false);
    assert_result!(crunchy.tenant_categories(options.clone()).await)
}

#[tokio::test]
async fn tenant_categories_with_subcategories() {
    let crunchy = SESSION.get().await.unwrap();
    let options = TenantCategoryOptions::default().include_subcategories(true);
    assert_result!(crunchy.tenant_categories(options.clone()).await)
}
