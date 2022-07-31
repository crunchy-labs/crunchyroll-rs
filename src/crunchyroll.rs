use std::collections::HashMap;
use serde::{Deserialize};
use std::fmt::{Display, Formatter};
use chrono::{DateTime, Utc};
use reqwest::RequestBuilder;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
#[allow(unused_imports)]
use smart_default::SmartDefault;
use crate::error::{check_request_error, CrunchyrollError, CrunchyrollErrorContext, Result};


#[derive(Copy, Clone, Debug)]
pub enum Locale {
    JP,US, LA, ES, FR, PT, BR, IT, DE, RU, AR
}

impl Display for Locale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let locale = match self {
            Locale::JP => "ja-JP",
            Locale::US => "en-US",
            Locale::LA => "es-419",
            Locale::ES => "es-ES",
            Locale::FR => "fr-FR",
            Locale::PT => "pt-PT",
            Locale::BR => "pt-BR",
            Locale::IT => "it-IT",
            Locale::DE => "de-DE",
            Locale::RU => "ru-RU",
            Locale::AR => "ar-SA"
        };
        write!(f, "{}", locale)
    }
}

#[derive(Debug)]
pub struct Crunchyroll {
    pub client: reqwest::Client,
    pub locale: Locale,

    pub config: CrunchyrollConfig,

    cache: bool
}

#[derive(Debug, Clone)]
pub struct CrunchyrollConfig {
    pub token_type: String,
    pub access_token: String,

    pub bucket: String,

    pub country_code: String,
    pub premium: bool,
    pub policy: String,
    pub key_pair_id: String,
    pub account_id: String,
    pub external_id: String,
}


#[derive(Deserialize, Debug)]
#[cfg_attr(not(all(test, feature = "__test_strict")), serde(default), derive(Default))]
#[allow(dead_code)]
struct LoginResponse {
    access_token: String,
    expires_in: i32,
    token_type: String,
    scope: String,
    country: String,
    account_id: String
}

/// This impl is only for the native login methods. Compiling to with wasm fails if every function
/// is in here because it don't know how to behave with `reqwest::Client`.
impl Crunchyroll {
    pub fn new() -> CrunchyrollBuilder {
        CrunchyrollBuilder {
            client: reqwest::Client::new(),
            locale: Locale::US
        }
    }

    async fn request<T: DeserializeOwned>(&self, mut builder: RequestBuilder) -> Result<T, CrunchyrollError> {
        builder = builder.
            bearer_auth(self.config.access_token.clone());

        request(builder).await
    }

    pub fn is_caching(&self) -> bool {
        return self.cache
    }

    pub fn set_caching(&mut self, caching: bool) {
        self.cache = caching
    }

    pub async fn invalidate_session(self) -> Result<()> {
        let endpoint = "https://crunchyroll.com/logout";
        self.request::<()>(self.client.get(endpoint)).await
    }
}

pub struct CrunchyrollBuilder {
    client: reqwest::Client,
    locale: Locale
}

impl CrunchyrollBuilder {
    pub fn client(&mut self, client: reqwest::Client) -> &Self {
        self.client = client;
        self
    }

    pub fn locale(&mut self, locale: Locale) -> &Self {
        self.locale = locale;
        self
    }

    /// Logs in with credentials (username or email and password) and returns a new `Crunchyroll` instance.
    pub async fn login_with_credentials(self, user: String, password: String) -> Result<Crunchyroll> {
        let endpoint = "https://beta-api.crunchyroll.com/auth/v1/token";
        let resp = self.client
            .post(endpoint)
            .header("Authorization", "Basic aHJobzlxM2F3dnNrMjJ1LXRzNWE6cHROOURteXRBU2Z6QjZvbXVsSzh6cUxzYTczVE1TY1k=")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(
                HashMap::from([
                    ("username", user.as_str()),
                    ("password", password.as_str()),
                    ("grant_type", "password")
                ])
            ).unwrap())
            .send()
            .await
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext{ message: e.to_string() }
            ))?;

        let resp_value = resp
            .json()
            .await
            .map_err(|e| CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: e.to_string() }
            ))?;

        self.post_login(check_request_error(resp_value)?).await
    }

    /// Logs in with a etp rt cookie and returns a new `Crunchyroll` instance.
    /// This cookie can be extracted if you activate crunchyroll beta and then copy the `etp_rt`
    /// cookie from your browser.
    /// Note that the cookie value changes all 24 hours or so.
    pub async fn login_with_etp_rt(self, etp_rt: String) -> Result<Crunchyroll> {
        let endpoint = "https://beta-api.crunchyroll.com/auth/v1/token";
        let resp = self.client
            .post(endpoint)
            .header("Authorization", "Basic bm9haWhkZXZtXzZpeWcwYThsMHE6")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("etp_rt={}", etp_rt))
            .body(serde_urlencoded::to_string(
                HashMap::from([
                    ("grant_type", "etp_rt_cookie")
                ])
            ).unwrap())
            .send()
            .await
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext{ message: e.to_string() }
            ))?;

        let resp_value = resp
            .json()
            .await
            .map_err(|e| CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: e.to_string() }
            ))?;

        self.post_login(check_request_error(resp_value)?).await
    }

    /// Logs in with a session id and returns a new `Crunchyroll` instance.
    /// The session id can be extracted if you log in to the crunchyroll website and copy the `session_id`
    /// cookie from your browser.
    /// This login method made some trouble in the past (login failed even though the session id was
    /// valid and the user logged in) and is therefore not very reliable.
    pub async fn login_with_session_id(self, session_id: String) -> Result<Crunchyroll> {
        let endpoint = format!("https://api.crunchyroll.com/start_session.0.json?session_id={}", session_id);
        let resp = self.client
            .get(endpoint)
            .send()
            .await
            .map_err(|e| CrunchyrollError::RequestError(
                CrunchyrollErrorContext{ message: e.to_string() }
            ))?;

        let mut etp_rt = None;
        for cookie in resp.cookies() {
            if cookie.name() == "etp_rt" {
                etp_rt = Some(cookie.value().to_string());
            }
        }

        if let Some(cookie) = etp_rt {
            self.login_with_etp_rt(cookie).await
        } else {
            Err(CrunchyrollError::LoginError(
                CrunchyrollErrorContext{ message: "invalid session id".into() }
            ))
        }
    }

    async fn post_login(self, login_response: LoginResponse) -> Result<Crunchyroll> {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", format!("{} {}", login_response.token_type, login_response.access_token).parse().unwrap());

        let index_endpoint = "https://beta-api.crunchyroll.com/index/v2";
        #[derive(Deserialize)]
        #[cfg_attr(not(all(test, feature = "__test_strict")), serde(default), derive(SmartDefault))]
        #[allow(dead_code)]
        struct IndexRespCms {
            bucket: String,
            #[cfg_attr(not(all(test, feature = "__test_strict")), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
            expires: DateTime<Utc>,
            key_pair_id: String,
            policy: String,
            signature: String
        }
        #[derive(Deserialize)]
        #[cfg_attr(all(test, feature = "__test_strict"), serde(deny_unknown_fields))]
        #[cfg_attr(not(all(test, feature = "__test_strict")), serde(default), derive(Default))]
        #[allow(dead_code)]
        struct IndexResp {
            cms: IndexRespCms,
            default_marketing_opt_in: bool,
            service_available: bool,

            #[cfg(all(test, feature = "__test_strict"))]
            cms_beta: crate::StrictValue
        }
        let index_req = self.client
            .get(index_endpoint)
            .headers(headers.to_owned());
        let index = request::<IndexResp>(index_req).await?;

        let me_endpoint = "https://beta-api.crunchyroll.com/accounts/v1/me";
        #[derive(Deserialize)]
        #[cfg_attr(not(all(test, feature = "__test_strict")), serde(default), derive(SmartDefault))]
        #[allow(dead_code)]
        struct MeResp {
            account_id: String,
            #[cfg_attr(not(all(test, feature = "__test_strict")), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
            created: DateTime<Utc>,
            email_verified: bool,
            external_id: String
        }
        let me_req = self.client
            .get(me_endpoint)
            .headers(headers.to_owned());
        let me = request::<MeResp>(me_req).await?;

        let config = CrunchyrollConfig{
            token_type: login_response.token_type,
            access_token: login_response.access_token,

            // '/' is trimmed so that urls which require it must be in .../{bucket}/... like format.
            // this just looks cleaner
            bucket: index.cms.bucket.strip_prefix("/").unwrap_or(index.cms.bucket.as_str()).to_string(),

            country_code: login_response.country,
            premium: index.cms.bucket.ends_with("crunchyroll"),
            policy: index.cms.policy,
            key_pair_id: index.cms.key_pair_id,
            account_id: login_response.account_id,
            external_id: me.external_id,
        };

        let crunchy = Crunchyroll{
            client: self.client,
            locale: self.locale,

            config,

            cache: true
        };

        Ok(crunchy)
    }
}

async fn request<T: DeserializeOwned>(builder: RequestBuilder) -> Result<T> {
    let resp = builder
        .send()
        .await
        .map_err(|e| CrunchyrollError::RequestError(
            CrunchyrollErrorContext{ message: e.to_string() }
        ))?;

    check_request_error(resp.json().await.map_err(|e| CrunchyrollError::DecodeError(
        CrunchyrollErrorContext{ message: e.to_string() }
    ))?)
}
