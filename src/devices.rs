use crate::common::BulkResult;
use crate::crunchyroll::Executor;
use crate::macros::enum_values;
use crate::{Crunchyroll, Result};
use chrono::{DateTime, Utc};
use crunchyroll_rs_internal::Request;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

enum_values! {
    pub enum DevicePlatformType {
        Mobile = "mobile"
        Web = "web"
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct DeviceLocation {
    pub country: String,
    pub city: String,
    pub area: String,
}

#[derive(Debug, Deserialize, Serialize, smart_default::SmartDefault, Request)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
pub struct Device {
    #[serde(skip)]
    pub(crate) executor: Arc<Executor>,

    pub id: String,
    pub account_id: String,
    pub profile_id: String,
    pub client_id: String,
    pub device_id: String,

    /// Human readable name of the device type, like 'Chrome on Windows' or 'Samsung Galaxy S23'
    pub device_type: String,
    /// Name of the device, if available.
    /// for example, if you have an iPhone 15, which is called 'My iPhone' (<- you can set this name
    /// on your phone as you wish), the `device_type` would be 'iPhone 15' whereas the `device_name`
    /// would be 'My iPhone'
    #[serde(deserialize_with = "crate::internal::serde::deserialize_empty_pre_string_to_none")]
    pub device_name: Option<String>,

    pub platform_type: DevicePlatformType,

    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub created: DateTime<Utc>,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub modified: DateTime<Utc>,

    /// Ip address where the device was last used
    pub ip: String,
    /// Location where the device was last used
    pub location: DeviceLocation,
    #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
    pub last_used: DateTime<Utc>,

    pub synced_offline_content: bool,
    pub deactivated: bool,

    /// If the device is currently used
    pub is_current: bool,
}

impl Device {
    /// Deactivates the current device (deletes the device session).
    pub async fn deactivate(&mut self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/{}/devices/{}/deactivate",
            self.account_id, self.device_id
        );
        self.executor.post(endpoint).request_raw(true).await?;

        self.deactivated = true;

        Ok(())
    }
}

impl Crunchyroll {
    /// Returns all devices where you are logged in.
    pub async fn active_devices(&self) -> Result<Vec<Device>> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/{}/devices/active",
            self.executor.details.account_id.clone()?
        );
        Ok(self
            .executor
            .get(endpoint)
            .request::<BulkResult<Device>>()
            .await?
            .items)
    }

    /// Activates a device with an code. Generally 6 characters long and used when logging in to non
    /// computer / smartphone devices like PlayStation, Xbox or Android TV.
    pub async fn activate_device(&self, code: String) -> Result<()> {
        let endpoint = "https://www.crunchyroll.com/auth/v1/device";
        self.executor
            .post(endpoint)
            .json(&json!({ "user_code": code }))
            .request_raw(true)
            .await?;
        Ok(())
    }

    /// Deactivates all devices (deletes all active sessions) besides the currently used one.
    pub async fn deactivate_all_devices(&self) -> Result<()> {
        let endpoint = format!(
            "https://www.crunchyroll.com/accounts/v1/{}/devices/deactivate",
            self.executor.details.account_id.clone()?
        );
        self.executor.post(endpoint).request_raw(true).await?;
        Ok(())
    }
}
