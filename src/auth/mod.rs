//! Login, session handling and the request execution for the [`Crunchyroll`](crate::Crunchyroll)
//! struct.
//!
//! # Login methods
//!
//! The following methods are available to create a session:
//! - [`CrunchyrollBuilder::login_with_etp_rt`]: login with the `etp_rt` cookie that is set when
//!   logging in via the browser. This is the recommended way to log in.
//! - [`CrunchyrollBuilder::login_with_oauth_code`]: SSO/OAuth flow via Crunchyroll's login page
//!   (see the `sso-login` example for a complete implementation).
//! - [`CrunchyrollBuilder::login_with_refresh_token`] and
//!   [`CrunchyrollBuilder::login_with_refresh_token_profile_id`]: re-login with a refresh token
//!   obtained from one of the other login methods.
//! - [`CrunchyrollBuilder::login_anonymously`]: login without an account.

mod builder;
mod executor;
mod login;
mod request;

use serde::{Deserialize, Serialize};

#[deprecated(since = "0.19.1", note = "Use `auth::platform_credentials` instead")]
pub mod app_credentials;
#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
pub mod middleware;
pub mod platform_credentials;

pub(crate) use executor::Executor;

pub use builder::CrunchyrollBuilder;

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
    /// Type of the device which issues the session, e.g. `Chrome on Windows` (matches the default
    /// web chrome user agent and [`CrunchyrollBuilder::platform`]), `ANDROIDTV`, `iPhone 15` or
    /// `SM-G980F` (Samsung Galaxy S20).
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
    TvAndroid,
    TvRoku,
    TvSamsung,
    TvLg,
    #[default]
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
