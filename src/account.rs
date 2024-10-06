//! Account specific actions.

use crate::{options, Crunchyroll, EmptyJsonProxy, Executor, Locale, Request, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Account data of the currently logged in user.
#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Account {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub username: String,
    pub email: String,
    pub phone: String,
    pub profile_name: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created: DateTime<Utc>,

    pub avatar: String,
    #[serde(deserialize_with = "crate::internal::serde::deserialize_wallpaper_from_id")]
    pub wallpaper: Wallpaper,

    pub account_id: String,
    pub external_id: String,

    #[serde(rename = "mature_content_flag_manga")]
    #[serde(deserialize_with = "mature_content_flag_manga")]
    pub manga_maturity_rating: MaturityRating,
    #[serde(rename = "maturity_rating")]
    pub video_maturity_rating: MaturityRating,

    #[serde(rename = "preferred_communication_language")]
    pub email_language: Locale,
    #[serde(rename = "preferred_content_audio_language")]
    pub preferred_audio_language: Locale,
    #[serde(rename = "preferred_content_subtitle_language")]
    pub preferred_subtitle_language: Locale,

    pub opt_out_free_trials: bool,
    pub opt_out_pm_updates: bool,
    #[serde(rename = "opt_out_store_deals")]
    pub email_store_details: bool,
    #[serde(rename = "opt_out_newsletters")]
    pub email_newsletter: bool,
    #[serde(rename = "opt_out_promotional_updates")]
    pub email_promotion_details: bool,

    pub cr_beta_opt_in: bool,
    pub qa_user: bool,
    pub email_verified: bool,
    pub has_password: bool,

    #[cfg(feature = "__test_strict")]
    crleg_email_verified: crate::StrictValue,
    #[cfg(feature = "__test_strict")]
    extended_maturity_rating: crate::StrictValue,
}

options! {
    /// Preferences which account details should be updates.
    UpdatePreferences;
    /// Updates the language in which emails are sent to your account.
    email_language(Locale, "preferred_communication_language") = None,
    /// Updates if store details should be sent to your email.
    email_store_details(bool, "opt_out_store_deals") = None,
    /// Updates if newsletters should be sent to your email.
    email_newsletter(bool, "opt_out_newsletters") = None,
    /// Updates if promotions for products and offers should be sent to your email.
    email_promotion_details(bool, "opt_out_promotional_updates") = None,
    /// Updates the language in which audio should be played.
    audio_language(Locale, "preferred_content_audio_language") = None,
    /// Updates the language in which subtitles should be shown if available.
    subtitle_language(Locale, "preferred_content_subtitle_language") = None,
    /// Updates if / how mature video content should be shown / be available. I do not know the use
    /// case of this tbh.
    mature_video_content(MaturityRating, "maturity_rating") = None,
    /// Updates if / how mature manga content should be shown / be available. I do not know the use
    /// case of this tbh.
    mature_manga_content(MaturityRating, "mature_content_flag_manga") = None
}

/// The [`Account`] struct is actually not required to perform this actions ([`Crunchyroll`] itself
/// would be enough) but to keep it clean it's only available here.
impl Account {
    /// Update preferences for your account.
    pub async fn update_preferences(&mut self, preferences: UpdatePreferences) -> Result<()> {
        let profile_endpoint = "https://www.crunchyroll.com/accounts/v1/me/profile";
        let notification_endpoint =
            "https://www.crunchyroll.com/accounts/v1/me/notification_settings";

        let mut updated_self = self.clone();

        let mut profile_update = serde_json::Map::new();
        let mut notification_update = serde_json::Map::new();

        if let Some(email_language) = preferences.email_language {
            profile_update.insert(
                "preferred_communication_language".into(),
                email_language.to_string().into(),
            );
            updated_self.preferred_subtitle_language = email_language;
        }
        if let Some(subtitle_language) = preferences.subtitle_language {
            profile_update.insert(
                "preferred_content_subtitle_language".into(),
                subtitle_language.to_string().into(),
            );
            updated_self.preferred_subtitle_language = subtitle_language;
        }
        if let Some(mature_video_content) = preferences.mature_video_content {
            profile_update.insert(
                "maturity_rating".into(),
                mature_video_content.to_string().into(),
            );
            updated_self.video_maturity_rating = mature_video_content;
        }
        if let Some(mature_manga_content) = preferences.mature_manga_content {
            profile_update.insert(
                "mature_content_flag_manga".into(),
                match &mature_manga_content {
                    MaturityRating::NotMature => "0".to_string().into(),
                    MaturityRating::Mature => "1".to_string().into(),
                    MaturityRating::Custom(custom) => custom.clone().into(),
                },
            );
            updated_self.manga_maturity_rating = mature_manga_content;
        }

        if let Some(email_store_details) = preferences.email_store_details {
            notification_update.insert("opt_out_store_deals".into(), email_store_details.into());
            updated_self.email_store_details = email_store_details;
        }
        if let Some(email_newsletter) = preferences.email_newsletter {
            notification_update.insert("opt_out_newsletters".into(), email_newsletter.into());
            updated_self.email_newsletter = email_newsletter;
        }
        if let Some(email_promotion_details) = preferences.email_promotion_details {
            notification_update.insert(
                "opt_out_promotional_updates".into(),
                email_promotion_details.into(),
            );
            updated_self.email_promotion_details = email_promotion_details
        }

        if !profile_update.is_empty() {
            self.executor
                .patch(profile_endpoint)
                .json(&Value::Object(profile_update))
                .request::<EmptyJsonProxy>()
                .await?;
        }
        if !notification_update.is_empty() {
            self.executor
                .patch(notification_endpoint)
                .json(&Value::Object(notification_update))
                .request::<EmptyJsonProxy>()
                .await?;
        }

        *self = updated_self;
        Ok(())
    }

    /// Changes the current account password.
    pub async fn change_password(
        &self,
        current_password: String,
        new_password: String,
    ) -> Result<()> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/credentials";
        self.executor
            .patch(endpoint)
            .json(&json!({
                "accountId": self.account_id.clone(),
                "current_password": current_password,
                "new_password": new_password,
            }))
            .request::<EmptyJsonProxy>()
            .await?;
        Ok(())
    }

    /// Changes the current account email.
    pub async fn change_email(&self, current_password: String, new_email: String) -> Result<()> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/credentials";
        self.executor
            .patch(endpoint)
            .json(&json!({
                "current_password": current_password,
                "new_email": new_email,
            }))
            .request_raw(true)
            .await?;
        Ok(())
    }

    /// Changes the current profile wallpaper.
    pub async fn change_wallpaper(&mut self, wallpaper: Wallpaper) -> Result<()> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/profile";
        self.executor
            .patch(endpoint)
            .json(&json!({"wallpaper": &wallpaper.id}))
            .request_raw(true)
            .await?;
        self.wallpaper = wallpaper;
        Ok(())
    }
}

impl Crunchyroll {
    /// Return information about the current account. [`Account`] can be used to modify account
    /// settings like the email or web interface language.
    pub async fn account(&self) -> Result<Account> {
        let mut result: serde_json::Map<String, Value> = serde_json::Map::new();

        let me_endpoint = "https://www.crunchyroll.com/accounts/v1/me";
        result.extend(
            self.executor
                .get(me_endpoint)
                .request::<HashMap<String, Value>>()
                .await?,
        );

        let profile_endpoint = "https://www.crunchyroll.com/accounts/v1/me/profile";
        result.extend(
            self.executor
                .get(profile_endpoint)
                .request::<HashMap<String, Value>>()
                .await?,
        );

        let notification_endpoint =
            "https://www.crunchyroll.com/accounts/v1/me/notification_settings";
        result.extend(
            self.executor
                .get(notification_endpoint)
                .request::<HashMap<String, Value>>()
                .await?,
        );

        let mut account: Account = serde_json::from_value(Value::Object(result))?;
        account.executor = self.executor.clone();

        Ok(account)
    }
}

fn mature_content_flag_manga<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<MaturityRating, D::Error> {
    let as_string = String::deserialize(deserializer)?;

    Ok(match as_string.as_str() {
        "0" => MaturityRating::NotMature,
        "1" => MaturityRating::Mature,
        _ => MaturityRating::Custom(as_string),
    })
}

mod wallpaper {
    use crate::{Crunchyroll, Request, Result};
    use serde::{Deserialize, Serialize};

    /// A collection of wallpapers under a specific title/topic.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct WallpaperCollection {
        pub title: String,
        pub assets: Vec<Wallpaper>,
    }

    /// Wallpaper which are shown at the top of your Crunchyroll profile.
    #[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct Wallpaper {
        pub id: String,
        pub title: String,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
    #[request(executor(items))]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct WallpaperResult {
        items: Vec<WallpaperCollection>,
    }

    impl Wallpaper {
        /// Returns all available wallpapers
        pub async fn all_wallpapers(crunchyroll: &Crunchyroll) -> Result<Vec<WallpaperCollection>> {
            let endpoint = format!(
                "https://www.crunchyroll.com/assets/v2/{}/wallpaper",
                crunchyroll.executor.details.locale
            );
            Ok(crunchyroll
                .executor
                .get(endpoint)
                .request::<WallpaperResult>()
                .await?
                .items)
        }

        /// Link to a low resolution image of the wallpaper.
        pub fn tiny_url(&self) -> String {
            format!(
                "https://static.crunchyroll.com/assets/wallpaper/360x115/{}",
                self.id
            )
        }

        pub fn medium_url(&self) -> String {
            format!(
                "https://static.crunchyroll.com/assets/wallpaper/720x180/{}",
                self.id
            )
        }

        /// Link to a high resolution image of the wallpaper.
        pub fn big_url(&self) -> String {
            format!(
                "https://static.crunchyroll.com/assets/wallpaper/1920x400/{}",
                self.id
            )
        }
    }
}

use crate::crunchyroll::MaturityRating;
pub use wallpaper::*;
