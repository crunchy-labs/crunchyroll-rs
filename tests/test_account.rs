use crate::utils::{SESSION, Store};
use crunchyroll_rs::Locale;
use crunchyroll_rs::account::{Account, UpdatePreferences, Wallpaper};
use crunchyroll_rs::crunchyroll::MaturityRating;

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
    let mut account = ACCOUNT.get().await.unwrap().clone();

    let old_preferences = UpdatePreferences::default()
        .email_language(account.email_language.clone())
        .email_store_details(account.email_store_details)
        .email_newsletter(account.email_newsletter)
        .email_promotion_details(account.email_promotion_details)
        .audio_language(account.preferred_audio_language.clone())
        .subtitle_language(account.preferred_subtitle_language.clone())
        .mature_video_content(account.video_maturity_rating.clone())
        .mature_manga_content(account.manga_maturity_rating.clone());
    let new_preferences = UpdatePreferences::default()
        .email_language(Locale::en_US)
        .email_store_details(!account.email_store_details)
        .email_newsletter(!account.email_newsletter)
        .email_promotion_details(!account.email_promotion_details)
        .audio_language(Locale::en_US)
        .subtitle_language(Locale::en_US)
        .mature_video_content(MaturityRating::Mature)
        .mature_manga_content(MaturityRating::Mature);

    assert_result!(account.update_preferences(new_preferences.clone()).await);
    assert_result!(account.update_preferences(old_preferences.clone()).await)
}

#[tokio::test]
async fn all_wallpapers() {
    let crunchy = SESSION.get().await.unwrap();

    assert_result!(Wallpaper::all_wallpapers(crunchy).await)
}
