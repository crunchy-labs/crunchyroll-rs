//! Credentials and user agents of the official Crunchyroll apps, fetched from the
//! [crunchy-labs/artifacts](https://github.com/crunchy-labs/artifacts) repository.

use crate::auth::platform_credentials::{
    PlatformCredentials, PlatformCredentialsDetails, get_platform_credentials,
};

#[deprecated(
    since = "0.19.1",
    note = "Use `auth::platform_credentials::PlatformCredentialsDetails` instead"
)]
pub type AppCredentialsPlatform = PlatformCredentialsDetails;
#[deprecated(
    since = "0.19.1",
    note = "Use `auth::platform_credentials::PlatformCredentials` instead"
)]
pub type AppCredentials = PlatformCredentials;

/// Fetches the credentials of some official Crunchyroll apps from the
/// [crunchy-labs/artifacts](https://github.com/crunchy-labs/artifacts) GitHub repository.
#[deprecated(
    since = "0.19.1",
    note = "Use `auth::platform_credentials::get_platform_credentials` instead"
)]
#[allow(deprecated)]
pub async fn get_app_credentials() -> Result<AppCredentials, reqwest::Error> {
    get_platform_credentials().await
}
