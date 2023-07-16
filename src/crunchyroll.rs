//! Builder and access to the [`Crunchyroll`] struct which is required to make any action.

use crate::enum_values;
use reqwest::Client;
use std::sync::Arc;

enum_values! {
    /// Enum of supported languages by Crunchyroll.
    #[allow(non_camel_case_types)]
    #[derive(Hash, Ord, PartialOrd)]
    pub enum Locale {
        ar_ME = "ar-ME"
        ar_SA = "ar-SA"
        de_DE = "de-DE"
        en_IN = "en-IN"
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
            Locale::en_IN,
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
            Locale::en_IN => "English (India)",
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
    /// Maturity rating.
    pub enum MaturityRating {
        NotMature = "M2"
        Mature = "M3"
    }
}

/// Starting point of this whole library.
#[derive(Clone, Debug)]
pub struct Crunchyroll {
    pub(crate) executor: Arc<Executor>,
}

impl Crunchyroll {
    pub fn builder() -> CrunchyrollBuilder {
        CrunchyrollBuilder::default()
    }

    /// Return the (cloned) [`Client`] which is internally used to make requests.
    pub fn client(&self) -> Client {
        self.executor.client.clone()
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
    use crate::error::{check_request, CrunchyrollError, CrunchyrollErrorContext};
    use crate::{Crunchyroll, Locale, Request, Result};
    use chrono::{DateTime, Duration, Utc};
    use http::header;
    use reqwest::{Client, ClientBuilder, IntoUrl, RequestBuilder};
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::ops::Add;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Stores if the refresh token or etp-rt cookie was used for login. Extract the token and use
    /// it as argument in their associated function ([`CrunchyrollBuilder::login_with_refresh_token`]
    /// or [`CrunchyrollBuilder::login_with_etp_rt`]) if you want to re-login into the account again.
    #[derive(Clone, Debug)]
    pub enum SessionToken {
        RefreshToken(String),
        EtpRt(String),
        Anonymous,
    }

    #[derive(Debug, Default, Deserialize)]
    #[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
    #[cfg_attr(not(feature = "__test_strict"), serde(default))]
    #[allow(dead_code)]
    struct AuthResponse {
        access_token: String,
        /// Is [`None`] if generated via [`Executor::auth_anonymously`].
        refresh_token: Option<String>,
        expires_in: i32,
        token_type: String,
        scope: String,
        country: String,
        /// Is [`None`] if generated via [`Executor::auth_anonymously`].
        account_id: Option<String>,
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
        pub(crate) preferred_audio_locale: Option<Locale>,

        pub(crate) bucket: String,

        pub(crate) premium: bool,
        pub(crate) signature: String,
        pub(crate) policy: String,
        pub(crate) key_pair_id: String,
        /// The account id is wrapped in a [`Result`] since [`Executor::auth_anonymously`] /
        /// [`CrunchyrollBuilder::login_anonymously`] doesn't return an account id and to prevent
        /// writing error messages multiple times in functions which require the account id to be
        /// set they can just get the id or return the fix set error message.
        pub(crate) account_id: Result<String>,
    }

    /// Contains which fixes should be used to make the api more reliable as Crunchyroll does weird
    /// stuff / delivers incorrect results.
    #[derive(Clone, Debug)]
    #[allow(unused)]
    pub(crate) struct ExecutorFixes {
        pub(crate) locale_name_parsing: bool,
        pub(crate) season_number: bool,
    }

    /// Internal struct to execute all request with.
    #[derive(Debug)]
    pub struct Executor {
        pub(crate) client: Client,

        /// Must be a mutex because `Executor` is always passed inside of `Arc` which does not allow
        /// direct changes to the struct.
        pub(crate) config: Mutex<ExecutorConfig>,
        pub(crate) details: ExecutorDetails,

        #[allow(unused)]
        pub(crate) fixes: ExecutorFixes,
    }

    impl Executor {
        pub(crate) fn get<U: IntoUrl>(self: &Arc<Self>, url: U) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), self.client.get(url))
        }

        pub(crate) fn post<U: IntoUrl>(self: &Arc<Self>, url: U) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), self.client.post(url))
        }

        pub(crate) fn put<U: IntoUrl>(self: &Arc<Self>, url: U) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), self.client.put(url))
        }

        pub(crate) fn patch<U: IntoUrl>(self: &Arc<Self>, url: U) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), self.client.patch(url))
        }

        pub(crate) fn delete<U: IntoUrl>(self: &Arc<Self>, url: U) -> ExecutorRequestBuilder {
            ExecutorRequestBuilder::new(self.clone(), self.client.delete(url))
        }

        pub(crate) async fn request<T: Request + DeserializeOwned>(
            self: &Arc<Self>,
            mut req: RequestBuilder,
        ) -> Result<T> {
            let mut config = self.config.lock().await;
            if config.session_expire <= Utc::now() {
                let login_response = match config.session_token.clone() {
                    SessionToken::RefreshToken(refresh_token) => {
                        Executor::auth_with_refresh_token(&self.client, refresh_token).await?
                    }
                    SessionToken::EtpRt(etp_rt) => {
                        Executor::auth_with_etp_rt(&self.client, etp_rt).await?
                    }
                    SessionToken::Anonymous => Executor::auth_anonymously(&self.client).await?,
                };

                let mut new_config = config.clone();
                new_config.token_type = login_response.token_type;
                new_config.access_token = login_response.access_token;
                new_config.session_token = match new_config.session_token {
                    SessionToken::RefreshToken(_) => {
                        SessionToken::RefreshToken(login_response.refresh_token.unwrap())
                    }
                    SessionToken::EtpRt(_) => {
                        SessionToken::EtpRt(login_response.refresh_token.unwrap())
                    }
                    SessionToken::Anonymous => SessionToken::Anonymous,
                };
                new_config.session_expire =
                    Utc::now().add(Duration::seconds(login_response.expires_in as i64));

                *config = new_config;
            }

            req = req.header(
                header::AUTHORIZATION,
                format!("Bearer {}", config.access_token),
            );
            req = req.header(header::CONTENT_TYPE, "application/json");

            let mut resp: T = request(&self.client, req).await?;

            // drop config here explicitly as `__set_executor` can call this function recursively
            // which would lead to a deadlock
            drop(config);

            resp.__set_executor(self.clone()).await;

            Ok(resp)
        }

        async fn auth_anonymously(client: &Client) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let resp = client
                .post(endpoint)
                .header(header::AUTHORIZATION, "Basic Y3Jfd2ViOg==")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(
                    serde_urlencoded::to_string([
                        ("grant_type", "client_id"),
                        ("scope", "offline_access"),
                    ])
                    .unwrap(),
                )
                .send()
                .await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_credentials(
            client: &Client,
            user: String,
            password: String,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let resp = client.post(endpoint)
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
                .send()
                .await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_refresh_token(
            client: &Client,
            refresh_token: String,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let resp = client.post(endpoint)
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
                .send()
                .await?;

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_etp_rt(client: &Client, etp_rt: String) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let resp = client
                .post(endpoint)
                .header(header::AUTHORIZATION, "Basic bm9haWhkZXZtXzZpeWcwYThsMHE6")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("etp_rt={etp_rt}"))
                .body(
                    serde_urlencoded::to_string([
                        ("grant_type", "etp_rt_cookie"),
                        ("scope", "offline_access"),
                    ])
                    .unwrap(),
                )
                .send()
                .await?;

            check_request(endpoint.to_string(), resp).await
        }
    }

    impl Default for Executor {
        fn default() -> Self {
            Self {
                client: Client::new(),
                config: Mutex::new(ExecutorConfig {
                    token_type: "".to_string(),
                    access_token: "".to_string(),
                    session_token: SessionToken::RefreshToken("".into()),
                    session_expire: Default::default(),
                }),
                details: ExecutorDetails {
                    locale: Default::default(),
                    preferred_audio_locale: None,
                    account_id: Ok("".to_string()),
                    bucket: "".to_string(),
                    premium: false,
                    signature: "".to_string(),
                    policy: "".to_string(),
                    key_pair_id: "".to_string(),
                },
                fixes: ExecutorFixes {
                    locale_name_parsing: false,
                    season_number: false,
                },
            }
        }
    }

    pub(crate) struct ExecutorRequestBuilder {
        executor: Arc<Executor>,
        builder: RequestBuilder,
    }

    impl ExecutorRequestBuilder {
        pub(crate) fn new(executor: Arc<Executor>, builder: RequestBuilder) -> Self {
            Self { executor, builder }
        }

        pub(crate) fn query<T: Serialize + ?Sized>(mut self, query: &T) -> ExecutorRequestBuilder {
            self.builder = self.builder.query(query);

            self
        }

        pub(crate) fn apply_locale_query(self) -> ExecutorRequestBuilder {
            let locale = self.executor.details.locale.clone();
            self.query(&[("locale", locale)])
        }

        pub(crate) fn apply_preferred_audio_locale_query(self) -> ExecutorRequestBuilder {
            if let Some(locale) = self.executor.details.preferred_audio_locale.clone() {
                self.query(&[("preferred_audio_language", locale)])
            } else {
                self
            }
        }

        pub(crate) fn json<T: Serialize + ?Sized>(mut self, json: &T) -> ExecutorRequestBuilder {
            self.builder = self.builder.json(json);

            self
        }

        pub(crate) async fn request<T: Request + DeserializeOwned>(self) -> Result<T> {
            self.executor.request(self.builder).await
        }

        pub(crate) async fn request_raw(self) -> Result<Vec<u8>> {
            Ok(self.builder.send().await?.bytes().await?.to_vec())
        }
    }

    /// A builder to construct a new [`Crunchyroll`] instance. To create it, call
    /// [`Crunchyroll::builder`].
    pub struct CrunchyrollBuilder {
        client: Client,
        locale: Locale,
        preferred_audio_locale: Option<Locale>,

        fixes: ExecutorFixes,
    }

    impl Default for CrunchyrollBuilder {
        fn default() -> Self {
            Self {
                client: CrunchyrollBuilder::predefined_client_builder()
                    .build()
                    .unwrap(),
                locale: Locale::en_US,
                preferred_audio_locale: None,
                fixes: ExecutorFixes {
                    locale_name_parsing: false,
                    season_number: false,
                },
            }
        }
    }

    impl CrunchyrollBuilder {
        /// Return a [`ClientBuilder`] which has all required configurations necessary to send
        /// successful requests to Crunchyroll, applied (most of the time; sometimes Crunchyroll has
        /// fluctuations that requests doesn't work for a specific amount of time and after that
        /// amount everything goes back to normal and works as it should). You can use this builder
        /// to configure the behavior of the download client. Use [`CrunchyrollBuilder::client`] or
        /// [`CrunchyrollBuilder::try_bypass`] to set your built client.
        pub fn predefined_client_builder() -> ClientBuilder {
            let mut root_store = rustls::RootCertStore::empty();
            root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
                |ta| {
                    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                },
            ));
            let tls_config = rustls::ClientConfig::builder()
                .with_cipher_suites(rustls::DEFAULT_CIPHER_SUITES)
                .with_kx_groups(&[&rustls::kx_group::X25519])
                .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
                .unwrap()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            Client::builder()
                .https_only(true)
                .cookie_store(true)
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36 Edg/114.0.1823.67a")
                .use_preconfigured_tls(tls_config)
        }

        /// Set a custom client that will be used in all api requests.
        /// It is recommended to use the client builder from
        /// [`CrunchyrollBuilder::predefined_client_builder`] as base as it has some configurations
        /// which may be needed to make successful requests to Crunchyroll.
        pub fn client(mut self, client: Client) -> CrunchyrollBuilder {
            self.client = client;
            self
        }

        /// Set in which languages all results which have human readable text in it should be
        /// returned.
        pub fn locale(mut self, locale: Locale) -> CrunchyrollBuilder {
            self.locale = locale;
            self
        }

        /// Set the audio language of media (like episodes) which should be returned when querying
        /// by any other method than the direct media id. For example, if the preferred audio locale
        /// were set to [`Locale::en_US`], the seasons queried with [`crate::Series::seasons`] would
        /// likely have [`Locale::en_US`] as their audio locale. This might not always work on all
        /// endpoints as Crunchyroll does Crunchyroll things (e.g. it seems to have no effect when
        /// changing the locale and using [`Crunchyroll::query`]).
        pub fn preferred_audio_locale(
            mut self,
            preferred_audio_locale: Locale,
        ) -> CrunchyrollBuilder {
            self.preferred_audio_locale = Some(preferred_audio_locale);
            self
        }

        /// Set season and episode locales by parsing the season name and check if it contains
        /// any language name.
        /// Under special circumstances, this can slow down some methods as additional request must
        /// be made. Currently, this applies to [`crate::Series`]. Whenever a request
        /// is made which returns [`crate::Series`], internally [`crate::Series::seasons`] is called
        /// for every series.
        /// See <https://github.com/crunchy-labs/crunchyroll-rs/issues/3> for more information.
        #[cfg(feature = "experimental-stabilizations")]
        #[cfg_attr(docsrs, doc(cfg(feature = "experimental-stabilizations")))]
        pub fn stabilization_locales(mut self, enable: bool) -> CrunchyrollBuilder {
            self.fixes.locale_name_parsing = enable;
            self
        }

        /// Set the season number of seasons by parsing a string which is delivered via the api too
        /// and looks to be more reliable than the actual integer season number Crunchyroll provides.
        #[cfg(feature = "experimental-stabilizations")]
        #[cfg_attr(docsrs, doc(cfg(feature = "experimental-stabilizations")))]
        pub fn stabilization_season_number(mut self, enable: bool) -> CrunchyrollBuilder {
            self.fixes.season_number = enable;
            self
        }

        /// Login without an account. This is just like if you would visit crunchyroll.com without
        /// an account. Some functions won't work if logged in with this method.
        pub async fn login_anonymously(self) -> Result<Crunchyroll> {
            let login_response = Executor::auth_anonymously(&self.client).await?;
            let session_token = SessionToken::Anonymous;

            self.post_login(login_response, session_token).await
        }

        /// Logs in with credentials (username or email and password) and returns a new `Crunchyroll`
        /// instance.
        pub async fn login_with_credentials<S: AsRef<str>>(
            self,
            user: S,
            password: S,
        ) -> Result<Crunchyroll> {
            let login_response = Executor::auth_with_credentials(
                &self.client,
                user.as_ref().to_string(),
                password.as_ref().to_string(),
            )
            .await?;
            let session_token =
                SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

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
            let login_response =
                Executor::auth_with_refresh_token(&self.client, refresh_token.as_ref().to_string())
                    .await?;
            let session_token =
                SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

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
                Executor::auth_with_etp_rt(&self.client, etp_rt.as_ref().to_string()).await?;
            let session_token = SessionToken::EtpRt(login_response.refresh_token.clone().unwrap());

            self.post_login(login_response, session_token).await
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

            let index_req = self.client.get(index_endpoint).header(
                header::AUTHORIZATION,
                format!(
                    "{} {}",
                    login_response.token_type, login_response.access_token
                ),
            );
            let index: IndexResp = request(&self.client, index_req).await?;

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
                        preferred_audio_locale: self.preferred_audio_locale,

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
                        account_id: login_response.account_id.ok_or_else(|| {
                            CrunchyrollError::Authentication(
                                "Login with a user account to use this function".into(),
                            )
                        }),
                    },
                    fixes: self.fixes,
                }),
            };

            Ok(crunchy)
        }
    }

    /// Make a request from the provided builder.
    async fn request<T: Request + DeserializeOwned>(
        client: &Client,
        req: RequestBuilder,
    ) -> Result<T> {
        let built_req = req.build()?;
        let url = built_req.url().to_string();
        let resp = client.execute(built_req).await?;

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
