use crate::enum_values;
use std::sync::Arc;

enum_values! {
    /// Enum of supported languages by Crunchyroll.
    #[allow(non_camel_case_types)]
    #[derive(Hash, Ord, PartialOrd)]
    pub enum Locale {
        ar_ME = "ar-ME"
        ar_SA = "ar-SA"
        de_DE = "de-DE"
        en_US = "en-US"
        es_419 = "es-419"
        es_ES = "es-ES"
        es_LA = "es-LA"
        fr_FR = "fr-FR"
        hi_IN = "hi-IN"
        it_IT = "it-IT"
        ja_JP = "ja-JP"
        pt_BR = "pt-BR"
        pt_PT = "pt-PT"
        ru_RU = "ru-RU"
        zh_CN = "zh-CN"
    }
}

impl Locale {
    pub fn all() -> Vec<Locale> {
        vec![
            Locale::ar_ME,
            Locale::ar_SA,
            Locale::de_DE,
            Locale::en_US,
            Locale::es_419,
            Locale::es_ES,
            Locale::es_LA,
            Locale::fr_FR,
            Locale::hi_IN,
            Locale::it_IT,
            Locale::ja_JP,
            Locale::pt_BR,
            Locale::pt_PT,
            Locale::ru_RU,
            Locale::zh_CN,
        ]
    }

    pub fn to_human_readable(&self) -> String {
        match self {
            Locale::ar_ME => "Arabic",
            Locale::ar_SA => "Arabic (Saudi Arabia)",
            Locale::de_DE => "German",
            Locale::en_US => "English (US)",
            Locale::es_419 | Locale::es_LA => "Spanish (Latin America)",
            Locale::es_ES => "Spanish (European)",
            Locale::fr_FR => "French",
            Locale::hi_IN => "Hindi",
            Locale::it_IT => "Italian",
            Locale::ja_JP => "Japanese",
            Locale::pt_BR => "Portuguese (Brazil)",
            Locale::pt_PT => "Portuguese (Europe)",
            Locale::ru_RU => "Russian",
            Locale::zh_CN => "Chinese (China)",
            Locale::Custom(custom) => custom.as_str(),
        }
        .to_string()
    }
}

enum_values! {
    pub enum MaturityRating {
        NotMature = "M2"
        Mature = "M3"
    }
}

pub(crate) const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:106.0) Gecko/20100101 Firefox/106.0";

/// Starting point of this whole library.
#[derive(Clone, Debug)]
pub struct Crunchyroll {
    pub(crate) executor: Arc<Executor>,
}

/// This impl is only for the native login methods. Compiling to with wasm fails if every function
/// is in here because it don't know how to behave with `reqwest::Client`.
impl Crunchyroll {
    pub fn builder() -> CrunchyrollBuilder {
        CrunchyrollBuilder::default()
    }

    /// Check if the current used account has premium.
    pub fn premium(&self) -> bool {
        self.executor.details.premium
    }

    /// Return the current session token. It can be used to log-in later with
    /// [`CrunchyrollBuilder::login_with_refresh_token`] or [`CrunchyrollBuilder::login_with_etp_rt`].
    pub async fn session_token(&self) -> SessionToken {
        self.executor.config.lock().await.session_token.clone()
    }
}

mod auth {
    use crate::crunchyroll::USER_AGENT;
    use crate::error::{check_request, CrunchyrollError, CrunchyrollErrorContext};
    use crate::{Crunchyroll, Locale, Request, Result};
    use chrono::{DateTime, Duration, Utc};
    use http::header;
    use isahc::config::Configurable;
    use isahc::tls::{ProtocolVersion, TlsConfigBuilder};
    use isahc::{AsyncReadResponseExt, HttpClient, HttpClientBuilder, ResponseExt};
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::ops::Add;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Stores if the refresh token or etp-rt cookie was used for login. Extract the token and use
    /// it as argument in their associated function ([`CrunchyrollBuilder::login_with_refresh_token`]
    /// or [`CrunchyrollBuilder::login_with_etp_rt`]) if you want to re-login into the account again.
    #[derive(Clone, Debug)]
    pub enum SessionToken {
        RefreshToken(String),
        EtpRt(String),
    }

    #[derive(Debug, Default, Deserialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
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

    #[derive(Clone, Debug)]
    pub(crate) struct ExecutorConfig {
        pub(crate) token_type: String,
        pub(crate) access_token: String,
        pub(crate) session_token: SessionToken,
        pub(crate) session_expire: DateTime<Utc>,
    }

    #[allow(dead_code)]
    #[derive(Clone, Debug)]
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
        pub(crate) client: HttpClient,

        // this must be a mutex because `Executor` is always passed inside of `Arc` which does not allow
        // direct changes to the struct
        pub(crate) config: Mutex<ExecutorConfig>,
        pub(crate) details: ExecutorDetails,
    }

    impl Executor {
        pub(crate) fn get<S: AsRef<str>>(self: &Arc<Self>, url: S) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), isahc::Request::get(url.as_ref()))
        }

        pub(crate) fn post<S: AsRef<str>>(self: &Arc<Self>, url: S) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), isahc::Request::post(url.as_ref()))
        }

        pub(crate) fn put<S: AsRef<str>>(self: &Arc<Self>, url: S) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), isahc::Request::put(url.as_ref()))
        }

        pub(crate) fn patch<S: AsRef<str>>(self: &Arc<Self>, url: S) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), isahc::Request::patch(url.as_ref()))
        }

        pub(crate) fn delete<S: AsRef<str>>(self: &Arc<Self>, url: S) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), isahc::Request::delete(url.as_ref()))
        }

        pub(crate) async fn request<T: Request + DeserializeOwned, B: Into<isahc::AsyncBody>>(
            self: &Arc<Self>,
            mut req: isahc::Request<B>,
        ) -> Result<T> {
            let mut config = self.config.lock().await;
            if config.session_expire <= Utc::now() {
                let login_response = match config.session_token.clone() {
                    SessionToken::RefreshToken(refresh_token) => {
                        Executor::auth_with_refresh_token(self.client.clone(), refresh_token)
                            .await?
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

            req.headers_mut().append(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&format!("Bearer {}", config.access_token)).unwrap(),
            );
            req.headers_mut().append(
                header::CONTENT_TYPE,
                header::HeaderValue::from_str("application/json").unwrap(),
            );

            let mut resp: T = request(self.client.clone(), req).await?;

            resp.__set_executor(self.clone());

            Ok(resp)
        }

        async fn auth_with_credentials(
            client: HttpClient,
            user: String,
            password: String,
        ) -> Result<AuthResponse> {
            Executor::pre_auth(&client).await?;

            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let req = isahc::Request::post(endpoint)
                .header(header::AUTHORIZATION, "Basic aHJobzlxM2F3dnNrMjJ1LXRzNWE6cHROOURteXRBU2Z6QjZvbXVsSzh6cUxzYTczVE1TY1k=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(
                    serde_urlencoded::to_string([
                        ("username", user.as_ref()),
                        ("password", password.as_ref()),
                        ("grant_type", "password"),
                        ("scope", "offline_access"),
                    ])
                    .unwrap(),
                )
                .unwrap();
            let resp = client.send_async(req).await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_refresh_token(
            client: HttpClient,
            refresh_token: String,
        ) -> Result<AuthResponse> {
            Executor::pre_auth(&client).await?;

            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let req = isahc::Request::post(endpoint)
                .header(header::AUTHORIZATION, "Basic aHJobzlxM2F3dnNrMjJ1LXRzNWE6cHROOURteXRBU2Z6QjZvbXVsSzh6cUxzYTczVE1TY1k=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(
                    serde_urlencoded::to_string([
                        ("refresh_token", refresh_token.as_str()),
                        ("grant_type", "refresh_token"),
                        ("scope", "offline_access"),
                    ])
                    .unwrap(),
                )
                .unwrap();
            let resp = client.send_async(req).await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_etp_rt(client: HttpClient, etp_rt: String) -> Result<AuthResponse> {
            Executor::pre_auth(&client).await?;

            let endpoint = "https://www.crunchyroll.com/auth/v1/token";

            let jar = client.cookie_jar().unwrap();
            jar.set(
                isahc::cookies::CookieBuilder::new("etp_rt", etp_rt)
                    .build()
                    .unwrap(),
                &http::Uri::from_str("https://www.crunchyroll.com/").unwrap(),
            )
            .unwrap();

            let req = isahc::Request::post(endpoint)
                .header(header::AUTHORIZATION, "Basic bm9haWhkZXZtXzZpeWcwYThsMHE6")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .cookie_jar(jar.clone())
                .body(
                    serde_urlencoded::to_string([
                        ("grant_type", "etp_rt_cookie"),
                        ("scope", "offline_access"),
                    ])
                    .unwrap(),
                )
                .unwrap();
            let resp = client.send_async(req).await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn pre_auth(client: &HttpClient) -> Result<()> {
            let mut resp = client.get_async("https://www.crunchyroll.com/").await?;

            if resp.status().as_u16() >= 400 {
                Err(CrunchyrollError::Internal(
                    CrunchyrollErrorContext::new("Failed to get index cookies")
                        .with_url("https://www.crunchyroll.com/")
                        .with_value(resp.bytes().await.unwrap().as_slice()),
                ))
            } else {
                Ok(())
            }
        }
    }

    impl Default for Executor {
        fn default() -> Self {
            Self {
                client: HttpClient::new().unwrap(),
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

    pub(crate) struct ExecutorRequestBuilder {
        executor: Arc<Executor>,
        builder: http::request::Builder,
        body: Option<Vec<u8>>,
    }

    impl ExecutorRequestBuilder {
        pub(crate) fn new(executor: Arc<Executor>, builder: http::request::Builder) -> Self {
            Self {
                executor,
                builder,
                body: None,
            }
        }

        pub(crate) fn query<K: Serialize + Sized, V: Serialize + Sized>(
            mut self,
            query: &[(K, V)],
        ) -> ExecutorRequestBuilder {
            let uri = self.builder.uri_ref().unwrap().clone();
            let path_and_query = uri.path_and_query().unwrap();
            let path = path_and_query.path();
            let query = if let Some(q) = path_and_query.query() {
                format!("{}&{}", q, serde_urlencoded::to_string(query).unwrap())
            } else {
                serde_urlencoded::to_string(query).unwrap()
            };
            self.builder = self.builder.uri(
                http::Uri::from_str(&format!(
                    "{}://{}{}?{}",
                    uri.scheme_str().unwrap(),
                    uri.host().unwrap(),
                    path,
                    query
                ))
                .unwrap(),
            );

            self
        }

        pub(crate) fn apply_media_query(self) -> ExecutorRequestBuilder {
            let details = self.executor.details.clone();

            self.query(&[
                ("Signature".to_string(), details.signature),
                ("Policy".to_string(), details.policy),
                ("Key-Pair-Id".to_string(), details.key_pair_id),
            ])
        }

        pub(crate) fn apply_locale_query(self) -> ExecutorRequestBuilder {
            let locale = self.executor.details.locale.clone();
            self.query(&[("locale", locale)])
        }

        pub(crate) fn json<T: Serialize + ?Sized>(mut self, json: &T) -> ExecutorRequestBuilder {
            self.body = Some(serde_json::to_vec(json).unwrap());

            self
        }

        pub(crate) async fn request<T: Request + DeserializeOwned>(self) -> Result<T> {
            if let Some(body) = self.body {
                self.executor
                    .request(self.builder.body(body).unwrap())
                    .await
            } else {
                self.executor.request(self.builder.body(()).unwrap()).await
            }
        }
    }

    /// A builder to construct a new [`Crunchyroll`] instance. To create it, call
    /// [`Crunchyroll::builder`].
    pub struct CrunchyrollBuilder {
        pub(crate) client: HttpClient,
        pub(crate) locale: Locale,
    }

    impl Default for CrunchyrollBuilder {
        fn default() -> Self {
            let tls = TlsConfigBuilder::default()
                .min_version(ProtocolVersion::Tlsv13)
                .build();
            let client = HttpClientBuilder::new()
                .default_header(header::USER_AGENT, USER_AGENT)
                .default_header(header::ACCEPT, "*")
                .proxy_tls_config(tls) // TODO: Change this to `tls_config` when https://github.com/sagebind/isahc/pull/388#discussion_r1014010929 is fixed
                .cookie_jar(isahc::cookies::CookieJar::new())
                .build()
                .unwrap();

            Self {
                client,
                locale: Locale::en_US,
            }
        }
    }

    impl CrunchyrollBuilder {
        /// Set a custom client over which all request to the api are made.
        /// Is it not recommended to overwrite the default client since its need a browser valid
        /// useragent and special ssl / tls config to bypass the Cloudflare Bot-Check used by
        /// Crunchyroll.
        pub fn client(&mut self, client: HttpClient) -> &Self {
            self.client = client;
            self
        }

        /// Set in which languages all results which have human readable text in it should be
        /// returned.
        pub fn locale(&mut self, locale: Locale) -> &Self {
            self.locale = locale;
            self
        }

        /// Logs in with credentials (username or email and password) and returns a new `Crunchyroll`
        /// instance.
        pub async fn login_with_credentials<S: AsRef<str>>(
            self,
            user: S,
            password: S,
        ) -> Result<Crunchyroll> {
            let login_response = Executor::auth_with_credentials(
                self.client.clone(),
                user.as_ref().to_string(),
                password.as_ref().to_string(),
            )
            .await?;
            let session_token = SessionToken::RefreshToken(login_response.refresh_token.clone());

            self.post_login(login_response, session_token).await
        }

        /// Logs in with a refresh token. This token is obtained when logging in with
        /// [`CrunchyrollBuilder::login_with_credentials`].
        /// Note: Even though the tokens used in [`CrunchyrollBuilder::login_with_refresh_token`] and
        /// [`CrunchyrollBuilder::login_with_etp_rt`] are having the same syntax, Crunchyroll
        /// internal they're different. I had issues when I tried to log in with the refresh token
        /// on [`CrunchyrollBuilder::login_with_etp_rt`] and vice versa.
        pub async fn login_with_refresh_token<S: AsRef<str>>(
            self,
            refresh_token: S,
        ) -> Result<Crunchyroll> {
            let login_response = Executor::auth_with_refresh_token(
                self.client.clone(),
                refresh_token.as_ref().to_string(),
            )
            .await?;
            let session_token = SessionToken::RefreshToken(login_response.refresh_token.clone());

            self.post_login(login_response, session_token).await
        }

        /// Logs in with a etp rt cookie and returns a new `Crunchyroll` instance.
        /// This cookie can be extracted if you copy the `etp_rt` cookie from your browser.
        /// Note: Even though the tokens used in [`CrunchyrollBuilder::login_with_etp_rt`] and
        /// [`CrunchyrollBuilder::login_with_refresh_token`] are having the same syntax, Crunchyroll
        /// internal they're different. I had issues when I tried to log in with the `etp_rt`
        /// cookie on [`CrunchyrollBuilder::login_with_refresh_token`] and vice versa.
        pub async fn login_with_etp_rt<S: AsRef<str>>(self, etp_rt: S) -> Result<Crunchyroll> {
            let login_response =
                Executor::auth_with_etp_rt(self.client.clone(), etp_rt.as_ref().to_string())
                    .await?;
            let session_token = SessionToken::EtpRt(login_response.refresh_token.clone());

            self.post_login(login_response, session_token).await
        }

        /// Logs in with a session id and returns a new `Crunchyroll` instance.
        /// The session id can be extracted if you log in to the crunchyroll website and copy the
        /// `session_id` cookie from your browser.
        /// This login method made some trouble in the past (login failed even though the session id was
        /// valid and the user logged in) and is therefore not very reliable.
        pub async fn login_with_session_id<S: AsRef<str>>(
            self,
            session_id: S,
        ) -> Result<Crunchyroll> {
            let endpoint = format!(
                "https://api.crunchyroll.com/start_session.0.json?session_id={}",
                session_id.as_ref()
            );
            let resp = self.client.get_async(&endpoint).await?;

            let jar = resp.cookie_jar().unwrap();
            let etp_rt = jar.get_by_name(
                &http::Uri::from_str("https://www.crunchyroll.com/").unwrap(),
                "etp_rt",
            );

            if let Some(cookie) = etp_rt {
                self.login_with_etp_rt(cookie.value()).await
            } else {
                Err(CrunchyrollError::Authentication(
                    CrunchyrollErrorContext::new("invalid session id").with_url(endpoint),
                ))
            }
        }

        async fn post_login(
            self,
            login_response: AuthResponse,
            session_token: SessionToken,
        ) -> Result<Crunchyroll> {
            let index_endpoint = "https://www.crunchyroll.com/index/v2";
            #[derive(Deserialize, smart_default::SmartDefault)]
            #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
            #[cfg_attr(not(feature = "__test_strict"), serde(default))]
            #[allow(dead_code)]
            struct IndexRespCms {
                bucket: String,
                #[default(DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH))]
                expires: DateTime<Utc>,
                key_pair_id: String,
                policy: String,
                signature: String,
            }
            #[derive(Default, Deserialize, Request)]
            #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
            #[cfg_attr(not(feature = "__test_strict"), serde(default))]
            #[allow(dead_code)]
            struct IndexResp {
                cms_web: IndexRespCms,
                default_marketing_opt_in: bool,
                service_available: bool,

                #[cfg(feature = "__test_strict")]
                cms: crate::StrictValue,
                #[cfg(feature = "__test_strict")]
                cms_beta: crate::StrictValue,
            }
            let index_req = isahc::Request::get(index_endpoint)
                .header(
                    header::AUTHORIZATION,
                    format!(
                        "{} {}",
                        login_response.token_type, login_response.access_token
                    ),
                )
                .body(())
                .unwrap();
            let index: IndexResp = request(self.client.clone(), index_req).await?;

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
                            .cms_web
                            .bucket
                            .strip_prefix('/')
                            .unwrap_or(index.cms_web.bucket.as_str())
                            .to_string(),

                        premium: false,
                        signature: index.cms_web.signature,
                        policy: index.cms_web.policy,
                        key_pair_id: index.cms_web.key_pair_id,
                        account_id: login_response.account_id,
                    },
                }),
            };

            Ok(crunchy)
        }
    }

    /// Make a request from the provided builder.
    async fn request<T: Request + DeserializeOwned, B: Into<isahc::AsyncBody>>(
        client: HttpClient,
        req: http::request::Request<B>,
    ) -> Result<T> {
        let url = req.uri().to_string();
        let resp = client.send_async(req).await?;

        #[cfg(not(feature = "__test_strict"))]
        {
            Ok(check_request(url, resp).await?)
        }
        #[cfg(feature = "__test_strict")]
        {
            let result = check_request(url.clone(), resp).await?;

            let cleaned = clean_request(result);
            let value = serde_json::Value::deserialize(serde::de::value::MapDeserializer::new(
                cleaned.into_iter(),
            ))?;
            serde_json::from_value(value.clone()).map_err(|e| {
                CrunchyrollError::Decode(
                    CrunchyrollErrorContext::new(format!("{} at {}:{}", e, e.line(), e.column()))
                        .with_url(url)
                        .with_value(value.to_string().as_bytes()),
                )
            })
        }
    }

    /// Removes all fields which are starting and ending with `__` from a map (which is usually the
    /// response of a request). Some fields can be excluded from this process by providing the field
    /// names in `not_clean_fields`.
    #[cfg(feature = "__test_strict")]
    fn clean_request(
        mut map: serde_json::Map<String, serde_json::Value>,
    ) -> serde_json::Map<String, serde_json::Value> {
        for (key, value) in map.clone() {
            if key.starts_with("__") && key.ends_with("__") {
                if key == "__links__" {
                    let classic_crunchyroll_exception: serde_json::Map<String, serde_json::Value> =
                        serde_json::from_value(value).unwrap();
                    #[allow(clippy::if_same_then_else)]
                    if classic_crunchyroll_exception.contains_key("episode/series")
                        && classic_crunchyroll_exception.contains_key("streams")
                    {
                        // `Episode` requires the __links__ field because crunchyroll does not provide another
                        // way to obtain a stream id
                        continue;
                    } else if map
                        .get("id")
                        .unwrap_or(&serde_json::Value::default())
                        .as_str()
                        .unwrap_or("")
                        .starts_with("dynamic_collection-")
                    {
                        // `HomeFeed` has some implementations which require __links__ to be accessible
                        continue;
                    }
                }
                map.remove(key.as_str());
            } else if let Some(object) = value.as_object() {
                map.insert(
                    key,
                    serde_json::to_value(clean_request(object.clone())).unwrap(),
                );
            } else if let Some(array) = value.as_array() {
                map.insert(
                    key,
                    serde_json::to_value(clean_request_array(array.clone())).unwrap(),
                );
            }
        }
        map
    }

    #[cfg(feature = "__test_strict")]
    fn clean_request_array(mut arr: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
        for (i, item) in arr.clone().iter().enumerate() {
            if let Some(object) = item.as_object() {
                arr[i] = serde_json::to_value(clean_request(object.clone())).unwrap();
            } else if let Some(array) = item.as_array() {
                arr[i] = serde_json::to_value(clean_request_array(array.clone())).unwrap();
            }
        }
        arr
    }
}

pub(crate) use auth::Executor;
pub use auth::{CrunchyrollBuilder, SessionToken};
