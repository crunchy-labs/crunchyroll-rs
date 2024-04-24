use crate::crunchyroll::MaturityRating;
use crate::macros::options;
use crate::{Crunchyroll, Executor, Locale, Result};
use crunchyroll_rs_internal::Request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

options! {
    /// Preferences which profile details should be updates.
    UpdateProfilePreferences;
    /// Updates the language in which audio should be played.
    audio_language(Locale, "preferred_content_audio_language") = None,
    /// Updates the language in which subtitles should be shown if available.
    subtitle_language(Locale, "preferred_content_subtitle_language") = None
}

/// An account profile.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Profile {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub username: String,
    pub email: String,

    pub profile_id: String,
    pub profile_name: String,

    pub can_switch: bool,
    pub is_primary: bool,
    pub is_selected: bool,

    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub wallpaper: String,

    pub maturity_rating: MaturityRating,

    pub preferred_communication_language: Option<Locale>,
    pub preferred_content_audio_language: Option<Locale>,
    pub preferred_content_subtitle_language: Option<Locale>,

    #[cfg(feature = "__test_strict")]
    account_id: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    do_not_sell: Option<crate::StrictValue>,
    #[cfg(feature = "__test_strict")]
    age_consent: Option<crate::StrictValue>,
}

impl Profile {
    /// Changes the profile name.
    pub async fn change_profile_name(&mut self, profile_name: String) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/me/multiprofile/{}",
            self.profile_id
        );
        let updated_self: Self = self
            .executor
            .patch(endpoint)
            .json(&json!({"profile_name": profile_name}))
            .request()
            .await?;

        self.profile_name = updated_self.profile_name;
        Ok(())
    }

    /// Updates some profile preferences.
    pub async fn update_preferences(
        &mut self,
        preferences: UpdateProfilePreferences,
    ) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/me/multiprofile/{}",
            self.profile_id
        );

        let mut updates = serde_json::Map::new();

        if let Some(audio_language) = preferences.audio_language {
            updates.insert(
                "preferred_content_audio_language".into(),
                audio_language.to_string().into(),
            );
        }
        if let Some(subtitle_language) = preferences.subtitle_language {
            updates.insert(
                "preferred_content_subtitle_language".into(),
                subtitle_language.to_string().into(),
            );
        }

        let updated_self: Self = self
            .executor
            .patch(endpoint)
            .json(&Value::Object(updates))
            .request()
            .await?;
        self.preferred_content_audio_language = updated_self.preferred_content_audio_language;
        self.preferred_content_subtitle_language = updated_self.preferred_content_subtitle_language;

        Ok(())
    }

    /// Updates if / how mature video content should be shown / be available. The password is
    /// required, else the request fails.
    pub async fn update_maturity_rating(
        &mut self,
        maturity_rating: MaturityRating,
        password: String,
    ) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/me/multiprofile/{}",
            self.profile_id
        );

        let updated_self: Self = self
            .executor
            .patch(endpoint)
            .json(&json!({"maturity_rating": maturity_rating, "password": password}))
            .request()
            .await?;
        self.maturity_rating = updated_self.maturity_rating;

        Ok(())
    }

    /// Deletes the current profile.
    pub async fn delete(self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/me/multiprofile/{}",
            self.profile_id
        );
        self.executor.delete(endpoint).request_raw(true).await?;
        Ok(())
    }
}

/// All profiles an account has.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[request(executor(profiles))]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Profiles {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub tier_max_profiles: u32,
    pub max_profiles: u32,

    pub profiles: Vec<Profile>,
}

impl Profiles {
    /// Creates a new profile. It is not checked if the maximum amount of profiles is already
    /// reached. Use [`Profiles::max_profiles`] and the length of [`Profiles::profiles`] to check it
    /// manually.
    pub async fn new(&self, profile_name: String, username: String) -> Result<Profile> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/multiprofile";
        self.executor
            .post(endpoint)
            .json(&json!({"profile_name": profile_name, "username": username}))
            .request()
            .await
    }
}

impl Crunchyroll {
    /// Returns the id of the currently used profile. Returns an empty string if logged in with
    /// [`crate::crunchyroll::CrunchyrollBuilder::login_anonymously`].
    pub async fn profile_id(&self) -> String {
        self.executor
            .jwt_claim::<String>("profile_id")
            .await
            .unwrap()
            .unwrap_or_default()
    }

    /// Requests all profiles the account has.
    pub async fn profiles(&self) -> Result<Profiles> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/multiprofile";
        self.executor.get(endpoint).request().await
    }
}
