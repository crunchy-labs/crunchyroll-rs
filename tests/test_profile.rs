use crate::utils::{Store, SESSION};
use crunchyroll_rs::crunchyroll::MaturityRating;
use crunchyroll_rs::profile::{Profiles, UpdateProfilePreferences};
use crunchyroll_rs::Locale;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;

mod utils;

static PROFILES: Store<Profiles> = Store::new(|| {
    Box::pin(async {
        let crunchy = SESSION.get().await?;
        let profiles = crunchy.profiles().await?;
        Ok(profiles)
    })
});

#[tokio::test]
async fn profiles() {
    assert_result!(PROFILES.get().await)
}

#[tokio::test]
async fn modify_profile() {
    let profiles = PROFILES.get().await.unwrap();

    let new_profile = profiles
        .new_profile(
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect(),
        )
        .await;
    assert_result!(new_profile);

    let mut profile = new_profile.unwrap();

    assert_result!(
        profile
            .change_profile_name(
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect()
            )
            .await
    );

    let preferences = UpdateProfilePreferences::default()
        .audio_language(Locale::en_US)
        .subtitle_language(Locale::en_US);
    assert_result!(profile.update_preferences(preferences.clone()).await);

    if let Ok(password) = env::var("PASSWORD") {
        assert_result!(
            profile
                .update_maturity_rating(MaturityRating::NotMature, password.clone())
                .await
        )
    }

    assert_result!(profile.clone().delete().await)
}
