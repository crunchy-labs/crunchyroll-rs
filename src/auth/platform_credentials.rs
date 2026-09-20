//! Credentials and user agents of some Crunchyroll apps, fetched from the
//! [crunchy-labs/artifacts](https://github.com/crunchy-labs/artifacts) repository.

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PlatformCredentialsDetails {
    pub client_id: String,
    pub client_secret: String,
    pub basic_auth_token: String,
    pub version: String,
    pub version_code: String,
}

/// Credentials of some Crunchyroll apps. Crunchyroll has more apps than the ones defined
/// here, but these are the ones that can be integrated relatively easy into the library.
#[derive(Deserialize, Serialize)]
pub struct PlatformCredentials {
    pub android_phone: PlatformCredentialsDetails,
    pub android_tv: PlatformCredentialsDetails,
}

impl PlatformCredentials {
    /// Returns the user agent of the Crunchyroll app for android phones. It should be used
    /// together with [`crate::auth::DevicePlatform::AndroidPhone`] via
    /// [`crate::auth::CrunchyrollBuilder::platform`].
    pub fn android_phone_user_agent(&self) -> String {
        format!(
            "Crunchyroll/{} Android/11 okhttp/5.3.2",
            self.android_phone.version
        )
    }

    /// Returns the user agent of the Crunchyroll app for android tv. It should be used
    /// together with [`crate::auth::DevicePlatform::TvAndroid`] via
    /// [`crate::auth::CrunchyrollBuilder::platform`].
    pub fn android_tv_user_agent(&self) -> String {
        format!(
            "Crunchyroll/ANDROIDTV/{}_{} (Android 13.0; en-US; TCL-S5400AF Build/TP1A.220624.014)",
            self.android_tv.version, self.android_tv.version_code
        )
    }
}

pub(super) const PLATFORM_CREDENTIALS_URL: &str =
    "https://raw.githubusercontent.com/crunchy-labs/artifacts/refs/heads/main/credentials.json";

/// Fetches the credentials of some Crunchyroll apps from the
/// [crunchy-labs/artifacts](https://github.com/crunchy-labs/artifacts) GitHub repository.
pub async fn get_platform_credentials() -> Result<PlatformCredentials, reqwest::Error> {
    reqwest::get(PLATFORM_CREDENTIALS_URL).await?.json().await
}
