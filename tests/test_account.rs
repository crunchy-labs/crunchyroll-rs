use crate::utils::{SESSION, Store};
use crunchyroll_rs::account::{Account, UpdateNotificationSettings};

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

#[tokio::test]
async fn account_update_preferences() {
    let account = ACCOUNT.get().await.unwrap().clone();
    let notification_settings = account.notification_settings().await.unwrap();

    let old_notification_settings = UpdateNotificationSettings::default()
        .newsletters(notification_settings.newsletters)
        .promotional_updates(notification_settings.promotional_updates)
        .store_deals(notification_settings.store_deals);
    let new_notification_settings = UpdateNotificationSettings::default()
        .newsletters(!notification_settings.newsletters)
        .promotional_updates(!notification_settings.promotional_updates)
        .store_deals(!notification_settings.store_deals);

    assert_result!(
        account
            .update_notification_settings(new_notification_settings.clone())
            .await
    );
    assert_result!(
        account
            .update_notification_settings(old_notification_settings.clone())
            .await
    )
}
