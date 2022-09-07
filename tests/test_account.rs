use crunchyroll_rs::{Account, Wallpaper};
use crate::utils::{SESSION, Store};

mod utils;

static ACCOUNT: Store<Account> = Store::new(|| Box::pin(async {
    let crunchy = SESSION.get().await?;
    let account = crunchy.account().await?;
    Ok(account)
}));

#[tokio::test]
async fn account() {
    let account = ACCOUNT.get().await;

    assert!(account.is_ok(), "{}", account.unwrap_err())
}

// More account testing is currently not possible because `ACCOUNT` must be mutable which is not
// implemented

#[tokio::test]
async fn all_wallpapers() {
    let crunchy = SESSION.get().await.unwrap();
    let wallpapers = Wallpaper::all_wallpapers(crunchy).await;

    assert!(wallpapers.is_ok(), "{}", wallpapers.unwrap_err())
}
