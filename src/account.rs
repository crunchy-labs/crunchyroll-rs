use crate::common::Request;
use crate::{Crunchyroll, Executor, Locale, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Debug, Deserialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Account {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub username: String,
    pub email: String,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created: DateTime<Utc>,

    pub avatar: String,
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
    #[serde(rename = "preferred_content_subtitle_language")]
    pub preferred_subtitle_language: Locale,

    pub opt_out_android_in_app_marketing: bool,
    pub opt_out_free_trials: bool,
    pub opt_out_new_media_queue_updates: bool,
    pub opt_out_newsletters: bool,
    pub opt_out_pm_updates: bool,
    pub opt_out_promotional_updates: bool,
    pub opt_out_queue_updates: bool,
    pub opt_out_store_deals: bool,

    pub cr_beta_opt_in: bool,
    pub qa_user: bool,
    pub email_verified: bool,

    #[cfg(feature = "__test_strict")]
    crleg_email_verified: crate::StrictValue,
}

/// The [`Account`] struct is actually not required to perform this actions ([`Crunchyroll`] itself
/// would be enough) but to keep it clean it's only available here.
impl Account {
    /// Updates the language in which emails are sent to your account.
    pub async fn update_email_language(&mut self, locale: Locale) -> Result<()> {
        self.update_preferences(
            "preferred_communication_language".into(),
            locale.to_string(),
        )
        .await?;
        self.email_language = locale;
        Ok(())
    }

    /// Updates the language in which subtitles should be shown if available.
    pub async fn update_preferred_subtitle_language(&mut self, locale: Locale) -> Result<()> {
        self.update_preferences(
            "preferred_content_subtitle_language".into(),
            locale.to_string(),
        )
        .await?;
        self.preferred_subtitle_language = locale;
        Ok(())
    }

    /// Updates if / how mature video content should be shown / be available. I do not know the use
    /// case of this tbh.
    pub async fn update_mature_video_content(
        &mut self,
        maturity_rating: MaturityRating,
    ) -> Result<()> {
        self.update_preferences("maturity_rating".into(), maturity_rating.to_string())
            .await?;
        self.video_maturity_rating = maturity_rating;
        Ok(())
    }

    /// Updates if / how mature manga content should be shown / be available. I do not know the use
    /// case of this tbh.
    pub async fn update_mature_manga_content(
        &mut self,
        maturity_rating: MaturityRating,
    ) -> Result<()> {
        self.update_preferences(
            "mature_content_flag_manga".into(),
            match &maturity_rating {
                MaturityRating::NotMature => "0".to_string(),
                MaturityRating::Mature => "1".to_string(),
                MaturityRating::Custom(custom) => custom.clone(),
            },
        )
        .await?;
        self.manga_maturity_rating = maturity_rating;
        Ok(())
    }

    async fn update_preferences(&self, name: String, value: String) -> Result<()> {
        let endpoint = "https://beta.crunchyroll.com/accounts/v1/me/profile";
        let builder = self.executor.client.patch(endpoint).json(&[(name, value)]);
        self.executor.request(builder).await
    }

    /// Changes the current account password.
    pub async fn change_password(
        &self,
        current_password: String,
        new_password: String,
    ) -> Result<()> {
        let endpoint = "https://beta.crunchyroll.com/accounts/v1/me/credentials";
        let builder = self.executor.client.patch(endpoint).json(&[
            ("accountId", self.account_id.clone()),
            ("current_password", current_password),
            ("new_password", new_password),
        ]);
        self.executor.request(builder).await
    }

    /// Changes the current account email.
    pub async fn change_email(&self, current_password: String, new_email: String) -> Result<()> {
        let endpoint = "https://beta.crunchyroll.com/accounts/v1/me/credentials";
        let builder = self.executor.client.patch(endpoint).json(&[
            ("current_password", current_password),
            ("new_email", new_email),
        ]);
        self.executor.request(builder).await
    }

    /// Changes the current profile wallpaper.
    pub async fn change_wallpaper(&mut self, wallpaper: Wallpaper) -> Result<()> {
        let endpoint = "https://beta.crunchyroll.com/accounts/v1/me/profile";
        let builder = self
            .executor
            .client
            .patch(endpoint)
            .json(&[("wallpaper", &wallpaper.name)]);
        self.executor.request(builder).await?;
        self.wallpaper = wallpaper;
        Ok(())
    }
}

impl Crunchyroll {
    /// Return information about the current account. [`Account`] can be used to modify account
    /// settings like the email or web interface language.
    pub async fn account(&self) -> Result<Account> {
        let mut result: HashMap<String, Value> = HashMap::new();

        let me_endpoint = "https://beta.crunchyroll.com/accounts/v1/me";
        let me_builder = self.executor.client.get(me_endpoint);
        result.extend(
            self.executor
                .request::<HashMap<String, Value>>(me_builder)
                .await?,
        );

        let profile_endpoint = "https://beta.crunchyroll.com/accounts/v1/me/profile";
        let profile_builder = self.executor.client.get(profile_endpoint);
        result.extend(
            self.executor
                .request::<HashMap<String, Value>>(profile_builder)
                .await?,
        );

        Ok(serde_json::from_value(serde_json::to_value(result)?)?)
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
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Default)]
    #[serde(from = "String")]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    pub struct Wallpaper {
        pub name: String,
    }

    #[derive(Debug, Deserialize, Default, Request)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    struct AllWallpapers {
        items: Vec<Wallpaper>,
    }

    impl From<String> for Wallpaper {
        fn from(s: String) -> Self {
            Self { name: s }
        }
    }

    impl Wallpaper {
        /// Returns all available wallpapers
        pub async fn all_wallpapers(crunchyroll: &Crunchyroll) -> Result<Vec<Wallpaper>> {
            let endpoint = "https://beta.crunchyroll.com/assets/v1/wallpaper";
            let builder = crunchyroll.executor.client.get(endpoint);
            let all_wallpapers: AllWallpapers = crunchyroll.executor.request(builder).await?;

            Ok(all_wallpapers.items)
        }

        /// Link to a low resolution image of the wallpaper.
        pub fn tiny_url(&self) -> String {
            format!(
                "https://static.crunchyroll.com/assets/wallpaper/360x115/{}",
                self.name
            )
        }

        /// Link to a high resolution image of the wallpaper.
        pub fn big_url(&self) -> String {
            format!(
                "https://static.crunchyroll.com/assets/wallpaper/1920x400/{}",
                self.name
            )
        }
    }
}

use crate::crunchyroll::MaturityRating;
pub use wallpaper::*;
