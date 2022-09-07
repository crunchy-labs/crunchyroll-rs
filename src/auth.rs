use crate::common::Request;
use crate::error::{check_request_error, CrunchyrollError, CrunchyrollErrorContext, Result};
use crate::{Crunchyroll, Locale};
use chrono::{DateTime, Duration, Utc};
use reqwest::header::HeaderMap;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Stores either the refresh token or etp-rt cookie used for internal login.
#[derive(Debug, Clone)]
pub enum SessionToken {
    RefreshToken(String),
    EtpRt(String),
}

#[derive(Deserialize, Debug)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default), derive(Default))]
#[allow(dead_code)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i32,
    token_type: String,
    scope: String,
    country: String,
    account_id: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ExecutorConfig {
    pub(crate) token_type: String,
    pub(crate) access_token: String,
    pub(crate) session_token: SessionToken,
    pub(crate) session_expire: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ExecutorDetails {
    pub(crate) locale: Locale,

    pub(crate) bucket: String,

    pub(crate) premium: bool,
    pub(crate) signature: String,
    pub(crate) policy: String,
    pub(crate) key_pair_id: String,
    pub(crate) account_id: String,
}

/// Internal struct to execute all request with.
#[derive(Debug)]
pub struct Executor {
    pub(crate) client: reqwest::Client,

    // this must be a mutex because `Executor` is always passed inside of `Arc` which does not allow
    // direct changes to the struct
    pub(crate) config: Mutex<ExecutorConfig>,
    pub(crate) details: ExecutorDetails,
}

impl Executor {
    pub(crate) async fn request<T: Request + DeserializeOwned>(
        self: &Arc<Self>,
        mut builder: RequestBuilder,
    ) -> Result<T, CrunchyrollError> {
        let mut config = self.config.lock().await;
        if config.session_expire <= Utc::now() {
            let login_response = match config.session_token.clone() {
                SessionToken::RefreshToken(refresh_token) => {
                    Executor::auth_with_refresh_token(self.client.clone(), refresh_token).await?
                }
                SessionToken::EtpRt(etp_rt) => {
                    Executor::auth_with_etp_rt(self.client.clone(), etp_rt).await?
                }
            };

            let mut new_config = config.clone();
            new_config.token_type = login_response.token_type;
            new_config.access_token = login_response.access_token;
            new_config.session_token = match new_config.session_token {
                SessionToken::RefreshToken(_) => {
                    SessionToken::RefreshToken(login_response.refresh_token)
                }
                SessionToken::EtpRt(_) => SessionToken::EtpRt(login_response.refresh_token),
            };
            new_config.session_expire =
                Utc::now().add(Duration::seconds(login_response.expires_in as i64));

            *config = new_config;
        }

        builder = builder.bearer_auth(config.access_token.clone());

        let mut resp: T = request(builder).await?;

        resp.__set_executor(self.clone());

        Ok(resp)
    }

    pub(crate) fn media_query(&self) -> Vec<(String, String)> {
        vec![
            ("locale".to_string(), self.details.locale.to_string()),
            ("Signature".to_string(), self.details.signature.clone()),
            ("Policy".to_string(), self.details.policy.clone()),
            ("Key-Pair-Id".to_string(), self.details.key_pair_id.clone()),
        ]
    }

    async fn auth_with_refresh_token(
        client: reqwest::Client,
        refresh_token: String,
    ) -> Result<AuthResponse> {
        let endpoint = "https://beta-api.crunchyroll.com/auth/v1/token";
        let resp = client
            .post(endpoint)
            .header(
                "Authorization",
                "Basic aHJobzlxM2F3dnNrMjJ1LXRzNWE6cHROOURteXRBU2Z6QjZvbXVsSzh6cUxzYTczVE1TY1k=",
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(
                serde_urlencoded::to_string(&[
                    ("refresh_token", refresh_token.as_str()),
                    ("grant_type", "refresh_token"),
                    ("scope", "offline_access"),
                ])
                .unwrap(),
            )
            .send()
            .await?;

        let resp_value = resp.bytes().await?;
        check_request_error(resp_value.as_ref())
    }

    async fn auth_with_etp_rt(client: reqwest::Client, etp_rt: String) -> Result<AuthResponse> {
        let endpoint = "https://beta-api.crunchyroll.com/auth/v1/token";
        let resp = client
            .post(endpoint)
            .header("Authorization", "Basic bm9haWhkZXZtXzZpeWcwYThsMHE6")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Cookie", format!("etp_rt={}", etp_rt))
            .body(
                serde_urlencoded::to_string(&[
                    ("grant_type", "etp_rt_cookie"),
                    ("scope", "offline_access"),
                ])
                .unwrap(),
            )
            .send()
            .await?;

        let resp_value = resp.bytes().await?;
        check_request_error(resp_value.as_ref())
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            client: Default::default(),
            config: Mutex::new(ExecutorConfig {
                token_type: "".to_string(),
                access_token: "".to_string(),
                session_token: SessionToken::RefreshToken("".into()),
                session_expire: Default::default(),
            }),
            details: ExecutorDetails {
                locale: Default::default(),
                account_id: "".to_string(),
                bucket: "".to_string(),
                premium: false,
                signature: "".to_string(),
                policy: "".to_string(),
                key_pair_id: "".to_string(),
            },
        }
    }
}

/// A builder to construct a new [`Crunchyroll`] instance. To create it, call [`Crunchyroll::new`].
pub struct CrunchyrollBuilder {
    pub(crate) client: reqwest::Client,
    pub(crate) locale: Locale,
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

    /// Logs in with credentials (username or email and password) and returns a new `Crunchyroll`
    /// instance.
    pub async fn login_with_credentials(
        self,
        user: String,
        password: String,
    ) -> Result<Crunchyroll> {
        let endpoint = "https://beta-api.crunchyroll.com/auth/v1/token";
        let resp = self
            .client
            .post(endpoint)
            .header(
                "Authorization",
                "Basic aHJobzlxM2F3dnNrMjJ1LXRzNWE6cHROOURteXRBU2Z6QjZvbXVsSzh6cUxzYTczVE1TY1k=",
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(
                serde_urlencoded::to_string(&[
                    ("username", user.as_str()),
                    ("password", password.as_str()),
                    ("grant_type", "password"),
                    ("scope", "offline_access"),
                ])
                .unwrap(),
            )
            .send()
            .await?;

        let resp_value = resp.bytes().await?;
        let login_response: AuthResponse = check_request_error(resp_value.as_ref())?;
        let session_token = SessionToken::RefreshToken(login_response.refresh_token.clone());

        self.post_login(login_response, session_token).await
    }

    /// Logs in with a refresh token.
    pub async fn login_with_refresh_token(self, refresh_token: String) -> Result<Crunchyroll> {
        let login_response =
            Executor::auth_with_refresh_token(self.client.clone(), refresh_token).await?;
        let session_token = SessionToken::RefreshToken(login_response.refresh_token.clone());

        self.post_login(login_response, session_token).await
    }

    /// Logs in with a etp rt cookie and returns a new `Crunchyroll` instance.
    /// This cookie can be extracted if you activate crunchyroll beta and then copy the `etp_rt`
    /// cookie from your browser.
    /// Note that the cookie value changes all 24 hours or so.
    pub async fn login_with_etp_rt(self, etp_rt: String) -> Result<Crunchyroll> {
        let login_response = Executor::auth_with_etp_rt(self.client.clone(), etp_rt).await?;
        let session_token = SessionToken::EtpRt(login_response.refresh_token.clone());

        self.post_login(login_response, session_token).await
    }

    /// Logs in with a session id and returns a new `Crunchyroll` instance.
    /// The session id can be extracted if you log in to the crunchyroll website and copy the
    /// `session_id` cookie from your browser.
    /// This login method made some trouble in the past (login failed even though the session id was
    /// valid and the user logged in) and is therefore not very reliable.
    pub async fn login_with_session_id(self, session_id: String) -> Result<Crunchyroll> {
        let endpoint = format!(
            "https://api.crunchyroll.com/start_session.0.json?session_id={}",
            session_id
        );
        let resp = self.client.get(endpoint).send().await?;

        let mut etp_rt = None;
        for cookie in resp.cookies() {
            if cookie.name() == "etp_rt" {
                etp_rt = Some(cookie.value().to_string());
            }
        }

        if let Some(cookie) = etp_rt {
            self.login_with_etp_rt(cookie).await
        } else {
            Err(CrunchyrollError::Authentication(
                CrunchyrollErrorContext::new("invalid session id".into()),
            ))
        }
    }

    async fn post_login(
        self,
        login_response: AuthResponse,
        session_token: SessionToken,
    ) -> Result<Crunchyroll> {
        let mut headers = HeaderMap::new();
        headers.append(
            "Authorization",
            format!(
                "{} {}",
                login_response.token_type, login_response.access_token
            )
            .parse()
            .unwrap(),
        );

        let index_endpoint = "https://beta-api.crunchyroll.com/index/v2";
        #[derive(Deserialize)]
        #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
        #[cfg_attr(
            not(feature = "__test_strict"),
            serde(default),
            derive(smart_default::SmartDefault)
        )]
        #[allow(dead_code)]
        struct IndexRespCms {
            bucket: String,
            #[cfg_attr(not(feature = "__test_strict"), default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH)))]
            expires: DateTime<Utc>,
            key_pair_id: String,
            policy: String,
            signature: String,
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
            cms_beta: crate::StrictValue,
            #[cfg(feature = "__test_strict")]
            cms_web: crate::StrictValue,
        }
        impl Request for IndexResp {}
        let index_req = self.client.get(index_endpoint).headers(headers.to_owned());
        let index = request::<IndexResp>(index_req).await?;

        let crunchy = Crunchyroll {
            executor: Arc::new(Executor {
                client: self.client,

                config: Mutex::new(ExecutorConfig {
                    token_type: login_response.token_type,
                    access_token: login_response.access_token,
                    session_token,
                    session_expire: Utc::now()
                        .add(Duration::seconds(login_response.expires_in as i64)),
                }),
                details: ExecutorDetails {
                    locale: self.locale,

                    // '/' is trimmed so that urls which require it must be in .../{bucket}/... like format.
                    // this just looks cleaner
                    bucket: index
                        .cms
                        .bucket
                        .strip_prefix('/')
                        .unwrap_or(index.cms.bucket.as_str())
                        .to_string(),

                    premium: false,
                    signature: index.cms.signature,
                    policy: index.cms.policy,
                    key_pair_id: index.cms.key_pair_id,
                    account_id: login_response.account_id,
                },
            }),
        };

        Ok(crunchy)
    }
}

/// Make a request from the provided builder.
async fn request<T: Request + DeserializeOwned>(builder: RequestBuilder) -> Result<T> {
    let resp = builder.send().await?;

    let result = check_request_error(resp.bytes().await?.as_ref())?;

    #[cfg(not(feature = "__test_strict"))]
    {
        Ok(result)
    }
    #[cfg(feature = "__test_strict")]
    {
        let cleaned = clean_request(result, T::__not_clean_fields());
        let value = serde_json::Value::deserialize(serde::de::value::MapDeserializer::new(
            cleaned.into_iter(),
        ))?;
        check_request_error(value.to_string().as_bytes())
    }
}

/// Removes all fields which are starting and ending with `__` from a map (which is usually the
/// response of a request). Some fields can be excluded from this process by providing the field
/// names in `not_clean_fields`.
#[cfg(feature = "__test_strict")]
fn clean_request(
    mut map: serde_json::Map<String, serde_json::Value>,
    not_clean_fields: Vec<String>,
) -> serde_json::Map<String, serde_json::Value> {
    for (key, value) in map.clone() {
        if key.starts_with("__") && key.ends_with("__") && !not_clean_fields.contains(&key) {
            map.remove(key.as_str());
        } else if let Some(object) = value.as_object() {
            map.insert(
                key,
                serde_json::to_value(clean_request(object.clone(), not_clean_fields.clone()))
                    .unwrap(),
            );
        } else if let Some(array) = value.as_array() {
            map.insert(
                key,
                serde_json::to_value(clean_request_array(array.clone(), not_clean_fields.clone()))
                    .unwrap(),
            );
        }
    }
    map
}

#[cfg(feature = "__test_strict")]
fn clean_request_array(
    mut arr: Vec<serde_json::Value>,
    not_clean_fields: Vec<String>,
) -> Vec<serde_json::Value> {
    for (i, item) in arr.clone().iter().enumerate() {
        if let Some(object) = item.as_object() {
            arr[i] = serde_json::to_value(clean_request(object.clone(), not_clean_fields.clone()))
                .unwrap();
        } else if let Some(array) = item.as_array() {
            arr[i] =
                serde_json::to_value(clean_request_array(array.clone(), not_clean_fields.clone()))
                    .unwrap();
        }
    }
    arr
}
