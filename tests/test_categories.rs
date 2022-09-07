use crate::utils::{Store, SESSION};
use crunchyroll_rs::categories::Category;
use crunchyroll_rs::BulkResult;

mod utils;

static CATEGORIES: Store<BulkResult<Category>> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        Ok(crunchy.categories().await?)
    })
});

#[tokio::test]
async fn categories() {
    assert_result!(CATEGORIES.get().await)
}
