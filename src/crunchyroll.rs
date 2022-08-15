use std::collections::HashMap;
use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use reqwest::RequestBuilder;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use crate::common::Request;
use crate::error::{check_request_error, CrunchyrollError, CrunchyrollErrorContext, Result};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Locale {
    JP,
    US,
    LA,
    ES,
    FR,
    PT,
    BR,
    IT,
    DE,
    RU,
    AR,
    Custom(String)
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
            Locale::AR => "ar-SA",
            Locale::Custom(raw) => raw
        };
        write!(f, "{}", locale)
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::Custom("".to_string())
    }
}

impl TryFrom<String> for Locale {
    type Error = CrunchyrollError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "ja-JP" => Ok(Locale::JP),
            "en-US" => Ok(Locale::US),
            "es-419" => Ok(Locale::LA),
            "es-ES" => Ok(Locale::ES),
            "fr-FR" => Ok(Locale::FR),
            "pt-PT" => Ok(Locale::PT),
            "pr-BR"=> Ok(Locale::BR),
            "it-IT" => Ok(Locale::IT),
            "de-DE" => Ok(Locale::DE),
            "ru-RU" => Ok(Locale::RU),
            "ar-SA" => Ok(Locale::AR),
            _ => Err(CrunchyrollError::DecodeError(
                CrunchyrollErrorContext{ message: format!("`{}` is not a valid locale", value) }
            ))
        }
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        crate::internal::serde::string_to_enum(deserializer)
    }
}

#[derive(Debug)]
pub(crate) struct Executor {
    pub(crate) client: reqwest::Client,
    pub(crate) locale: Locale,

    pub(crate) config: CrunchyrollConfig
}

impl Executor {
    pub(crate) async fn request<T: Request>(self: &Arc<Self>, mut builder: RequestBuilder) -> Result<T, CrunchyrollError> {
        builder = builder.
            bearer_auth(self.config.access_token.clone());

        let mut resp: T = request(builder).await?;

        resp.set_executor(self.clone());

        Ok(resp)
    }

    pub(crate) fn media_query(&self) -> HashMap<String, String> {
        let mut query = HashMap::new();
        query.insert("locale".to_string(), self.locale.to_string());
        query.insert("Signature".to_string(), self.config.signature.clone());
        query.insert("Policy".to_string(), self.config.policy.clone());
        query.insert("Key-Pair-Id".to_string(), self.config.key_pair_id.clone());

        query
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            client: Default::default(),
            locale: Locale::JP,
            config: CrunchyrollConfig {
                token_type: "".to_string(),
                access_token: "".to_string(),
                refresh_token: "".to_string(),
                bucket: "".to_string(),
                country_code: "".to_string(),
                premium: false,
                signature: "".to_string(),
                policy: "".to_string(),
                key_pair_id: "".to_string(),
                account_id: "".to_string(),
                external_id: "".to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Crunchyroll {
    pub(crate) executor: Arc<Executor>
}

#[derive(Debug, Clone)]
pub struct CrunchyrollConfig {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,

    pub bucket: String,

    pub country_code: String,
    pub premium: bool,
    pub signature: String,
    pub policy: String,
    pub key_pair_id: String,
    pub account_id: String,
    pub external_id: String,
}


#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
#[allow(dead_code)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
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

    pub fn config(&self) -> CrunchyrollConfig {
        self.executor.config.clone()
    }

    pub async fn invalidate_session(self) -> Result<()> {
        let endpoint = "https://crunchyroll.com/logout";
        self.executor.to_owned().request(self.executor.client.get(endpoint)).await
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
                    ("grant_type", "password"),
                    ("scope", "offline_access")
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
                    ("grant_type", "etp_rt_cookie"),
                    ("scope", "offline_access")
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
            Err(CrunchyrollError::AuthenticationError(
                CrunchyrollErrorContext{ message: "invalid session id".into() }
            ))
        }
    }

    async fn post_login(self, login_response: LoginResponse) -> Result<Crunchyroll> {
        let mut headers = HeaderMap::new();
        headers.append("Authorization", format!("{} {}", login_response.token_type, login_response.access_token).parse().unwrap());

        let index_endpoint = "https://beta-api.crunchyroll.com/index/v2";
        #[derive(Deserialize)]
        #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
        #[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
        #[allow(dead_code)]
        struct IndexRespCms {
            bucket: String,
            #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
            expires: DateTime<Utc>,
            key_pair_id: String,
            policy: String,
            signature: String
        }
        #[derive(Deserialize)]
        #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
        #[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
        #[allow(dead_code)]
        struct IndexResp {
            cms: IndexRespCms,
            default_marketing_opt_in: bool,
            service_available: bool,

            #[cfg(feature = "__test_strict")]
            cms_beta: crate::StrictValue
        }
        let index_req = self.client
            .get(index_endpoint)
            .headers(headers.to_owned());
        let index = request::<IndexResp>(index_req).await?;

        let me_endpoint = "https://beta-api.crunchyroll.com/accounts/v1/me";
        #[derive(Deserialize)]
        #[cfg_attr(not(feature = "__test_strict"), serde(default), derive(smart_default::SmartDefault))]
        #[allow(dead_code)]
        struct MeResp {
            account_id: String,
            #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
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
            refresh_token: login_response.refresh_token,

            // '/' is trimmed so that urls which require it must be in .../{bucket}/... like format.
            // this just looks cleaner
            bucket: index.cms.bucket.strip_prefix("/").unwrap_or(index.cms.bucket.as_str()).to_string(),

            country_code: login_response.country,
            premium: index.cms.bucket.ends_with("crunchyroll"),
            signature: index.cms.signature,
            policy: index.cms.policy,
            key_pair_id: index.cms.key_pair_id,
            account_id: login_response.account_id,
            external_id: me.external_id,
        };

        let crunchy = Crunchyroll{
            executor: Arc::new(Executor {
                client: self.client,
                locale: self.locale,

                config
            }),
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

    let result = check_request_error(resp.json().await.map_err(|e| CrunchyrollError::DecodeError(
        CrunchyrollErrorContext{ message: e.to_string() }
    ))?)?;

    #[cfg(not(feature = "__test_strict"))]
    return Ok(result);
    #[cfg(feature = "__test_strict")]
    {
        let cleaned = clean_request(result);
        return T::deserialize(serde::de::value::MapDeserializer::new(cleaned.into_iter())).map_err(|e| CrunchyrollError::DecodeError(
            CrunchyrollErrorContext { message: e.to_string() }
        ));
    }
}

#[cfg(feature = "__test_strict")]
fn clean_request(mut map: serde_json::Map<String, serde_json::Value>) -> serde_json::Map<String, serde_json::Value> {
    for (key, value) in map.clone() {
        if key.starts_with("__") && key.ends_with("__") {
            map.remove(key.as_str());
        } else if let Some(object) = value.as_object() {
            map.insert(key, serde_json::to_value(clean_request(object.clone())).unwrap());
        }
    }
    map
}
