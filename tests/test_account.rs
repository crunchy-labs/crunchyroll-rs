use crate::utils::{SESSION, Store};
use crunchyroll_rs::account::Account;

mod utils;

static ACCOUNT: Store<Account> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let account = crunchy.account().await?;
        Ok(account)
    })
});

#[tokio::test]
async fn account() {
    assert_result!(ACCOUNT.get().await)
}
