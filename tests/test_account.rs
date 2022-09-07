use crate::utils::{Store, SESSION};
use crunchyroll_rs::account::{Account, Wallpaper};

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

// More account testing is currently not possible because `ACCOUNT` must be mutable which is not
// implemented

#[tokio::test]
async fn all_wallpapers() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(Wallpaper::all_wallpapers(crunchy).await)
}
