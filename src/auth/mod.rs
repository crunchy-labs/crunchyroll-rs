//! Login, session handling and the request execution for the [`Crunchyroll`](crate::Crunchyroll)
//! struct.

mod builder;
mod executor;
mod login;
mod request;

pub mod app_credentials;
pub use builder::CrunchyrollBuilder;
pub(crate) use executor::Executor;

use serde::{Deserialize, Serialize};

/// Stores if the refresh token or etp-rt cookie was used for login. Extract the token and use
/// it as argument in their associated function ([`CrunchyrollBuilder::login_with_refresh_token`]
/// or [`CrunchyrollBuilder::login_with_etp_rt`]) if you want to re-login into the account again.
#[derive(Clone, Debug)]
pub enum SessionToken {
    RefreshToken(String),
    EtpRt(String),
    Anonymous,
}

/// Information about the device that creates a new session.
#[derive(Clone, Debug)]
pub struct DeviceIdentifier {
    /// The device id, this is specific for every device type, but usually represented as UUID.
    /// Using [`uuid::Uuid::new_v4`] for it works fine.
    pub device_id: String,
    /// Type of the device which issues the session, e.g. `ANDROIDTV` (recommended, this is on
    /// par with the default user agent and [`CrunchyrollBuilder::platform`]),
    /// `Chrome on Windows`, `iPhone 15` or `SM-G980F` (Samsung Galaxy S20).
    pub device_type: String,
    /// Name of the device which issues the session. This may be empty, for example all session
    /// that are created over the website have an empty name; when issues via the app, the name
    /// is the name of your phone (which you can modify/set when you set up the phone).
    pub device_name: Option<String>,
}

impl Default for DeviceIdentifier {
    fn default() -> Self {
        Self {
            device_id: "0000-0000-0000-0000".to_string(),
            device_type: "0000-0000-0000-0000".to_string(),
            device_name: None,
        }
    }
}

/// Platforms that can request a [`Stream`](crate::media::Stream). Because not all platforms have their own variant, use
/// [`DevicePlatform::Custom`] to define one.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum DevicePlatform {
    AndroidPhone,
    AndroidTablet,
    ConsolePs4,
    ConsolePs5,
    ConsoleSwitch,
    ConsoleXboxOne,
    IosIpad,
    IosIphone,
    IosVision,
    #[default]
    TvAndroid,
    TvRoku,
    TvSamsung,
    TvLg,
    WebChrome,
    WebEdge,
    WebFirefox,
    WebSafari,
    Custom {
        /// A device, e.g. `tv` or `web`.
        device: String,
        /// A platform, e.g. `roku` or `chrome`.
        platform: String,
    },
}
