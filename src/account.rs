//! Account specific actions.

use crate::{Crunchyroll, EmptyJsonProxy, Executor, Locale, Request, Result, options};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// Account data of the current user.
#[derive(Clone, Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Account {
    #[serde(skip)]
    executor: Arc<Executor>,

    pub account_id: String,
    pub external_id: String,

    pub email: String,
    pub phone: String,

    pub email_verified: bool,
    pub has_password: bool,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct NotificationSettings {
    #[serde(rename = "opt_out_free_trials")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    pub free_trials: bool,
    #[serde(rename = "opt_out_newsletters")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    pub newsletters: bool,
    #[serde(rename = "opt_out_pm_updates")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    pub pm_updates: bool,
    #[serde(rename = "opt_out_promotional_updates")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    pub promotional_updates: bool,
    #[serde(rename = "opt_out_store_deals")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    pub store_deals: bool,
    // at the time of writing, the following two entries aren't available on all accounts
    #[serde(rename = "opt_out_new_media_queue_updates")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    #[serde(default)]
    pub media_queue_updates: bool,
    #[serde(rename = "opt_out_whats_app")]
    #[serde(deserialize_with = "crate::internal::serde::deserialize_bool_invert")]
    #[serde(default)]
    pub whatsapp: bool,
}

options! {
    /// Preferences which account details should be updates.
    UpdateNotificationSettings;
    /// Updates the language in which emails are sent to your account.
    communication_language(Locale, "preferred_communication_language") = None,
    /// Updates if newsletters should be sent to your email.
    newsletters(bool, "opt_out_newsletters") = None,
    /// Updates if promotions for products and offers should be sent to your email.
    promotional_updates(bool, "opt_out_promotional_updates") = None,
    /// Updates if store details should be sent to your email.
    store_deals(bool, "opt_out_store_deals") = None
}

/// The [`Account`] struct is actually not required to perform this actions ([`Crunchyroll`] itself
/// would be enough) but to keep it clean it's only available here.
impl Account {
    /// Get the notification settings.
    pub async fn notification_settings(&self) -> Result<NotificationSettings> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me/notification_settings";
        self.executor.get(endpoint).request().await
    }

    /// Updates the notification settings.
    pub async fn update_notification_settings(
        &self,
        mut notification_settings: UpdateNotificationSettings,
    ) -> Result<()> {
        let profile_endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/me/multiprofile/{}",
            self.account_id
        );
        let notification_endpoint =
            "https://www.crunchyroll.com/accounts/v1/me/notification_settings";

        if let Some(communication_language) = notification_settings.communication_language {
            self.executor
                .patch(profile_endpoint)
                .json(&[(
                    "preferred_communication_language",
                    communication_language.to_string(),
                )])
                .request_raw(true)
                .await?;
            notification_settings.communication_language = None;
        }

        self.executor
            .patch(notification_endpoint)
            .json(&notification_settings.into_json())
            .request::<EmptyJsonProxy>()
            .await?;
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
}

impl Crunchyroll {
    /// Return information about the current account. [`Account`] can be used to modify account
    /// settings like the email or web interface language.
    pub async fn account(&self) -> Result<Account> {
        let endpoint = "https://www.crunchyroll.com/accounts/v1/me";
        self.executor.get(endpoint).request().await
    }
}
