use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppCredentialsPlatform {
    pub client_id: String,
    pub client_secret: String,
    pub basic_auth_token: String,
    pub version: String,
    pub version_code: String,
}

#[derive(Deserialize)]
pub struct AppCredentials {
    pub android_phone: AppCredentialsPlatform,
    pub android_tv: AppCredentialsPlatform,
}

impl AppCredentials {
    pub fn android_phone_user_agent(&self) -> String {
        format!(
            "Crunchyroll/{} Android/11 okhttp/5.3.2",
            self.android_phone.version
        )
    }

    pub fn android_tv_user_agent(&self) -> String {
        format!(
            "Crunchyroll/ANDROIDTV/{}_{} (Android 13.0; en-US; TCL-S5400AF Build/TP1A.220624.014)",
            self.android_tv.version, self.android_tv.version_code
        )
    }
}

pub(super) const APP_CREDENTIALS_URL: &str =
    "https://raw.githubusercontent.com/crunchy-labs/artifacts/refs/heads/main/credentials.json";

pub async fn get_app_credentials() -> Result<AppCredentials, reqwest::Error> {
    reqwest::get(APP_CREDENTIALS_URL).await?.json().await
}
