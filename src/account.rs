//! Account specific actions.

use crate::{Crunchyroll, EmptyJsonProxy, Executor, Request, Result};
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

/// The [`Account`] struct is actually not required to perform this actions ([`Crunchyroll`] itself
/// would be enough) but to keep it clean it's only available here.
impl Account {
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
