use crate::utils::{Store, SESSION};
use crunchyroll_rs::categories::TenantCategory;
use crunchyroll_rs::BulkResult;

mod utils;

static TENANT_CATEGORIES: Store<BulkResult<TenantCategory>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(crunchy.tenant_categories().await?)
    })
});

#[tokio::test]
async fn tenant_categories() {
    assert_result!(TENANT_CATEGORIES.get().await)
}
