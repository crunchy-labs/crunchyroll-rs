//! Builder and access to the [`Crunchyroll`] struct which is required to make any action.

use crate::enum_values;
use reqwest::Client;
use std::sync::Arc;

enum_values! {
    /// Enum of supported languages by Crunchyroll.
    /// Crunchyroll lists the available languages in the following api results:
    /// - <https://static.crunchyroll.com/config/i18n/v3/audio_languages.json>
    /// - <https://static.crunchyroll.com/config/i18n/v3/timed_text_languages.json>
    #[allow(non_camel_case_types)]
    #[derive(Hash, Ord, PartialOrd)]
    pub enum Locale {
        ar_ME = "ar-ME"
        ar_SA = "ar-SA"
        ca_ES = "ca-ES"
        de_DE = "de-DE"
        en_IN = "en-IN"
        en_US = "en-US"
        es_419 = "es-419"
        es_ES = "es-ES"
        es_LA = "es-LA"
        fr_FR = "fr-FR"
        hi_IN = "hi-IN"
        id_ID = "id-ID"
        it_IT = "it-IT"
        ja_JP = "ja-JP"
        ko_KR = "ko-KR"
        ms_MY = "ms-MY"
        pl_PL = "pl-PL"
        pt_BR = "pt-BR"
        pt_PT = "pt-PT"
        ru_RU = "ru-RU"
        ta_IN = "ta-IN"
        te_IN = "te-IN"
        th_TH = "th-TH"
        tr_TR = "tr-TR"
        vi_VN = "vi-VN"
        zh_CN = "zh-CN"
        zh_HK = "zh_HK"
        zh_TW = "zh-TW"
    }
}

impl Locale {
    pub fn all() -> Vec<Locale> {
        vec![
            Locale::ar_ME,
            Locale::ar_SA,
            Locale::ca_ES,
            Locale::de_DE,
            Locale::en_IN,
            Locale::en_US,
            Locale::es_419,
            Locale::es_ES,
            Locale::es_LA,
            Locale::fr_FR,
            Locale::hi_IN,
            Locale::id_ID,
            Locale::it_IT,
            Locale::ja_JP,
            Locale::ko_KR,
            Locale::ms_MY,
            Locale::pl_PL,
            Locale::pt_BR,
            Locale::pt_PT,
            Locale::ru_RU,
            Locale::ta_IN,
            Locale::te_IN,
            Locale::th_TH,
            Locale::tr_TR,
            Locale::vi_VN,
            Locale::zh_CN,
            Locale::zh_CN,
            Locale::zh_TW,
        ]
    }

    pub fn to_human_readable(&self) -> String {
        match self {
            Locale::ar_ME => "Arabic",
            Locale::ar_SA => "Arabic (Saudi Arabia)",
            Locale::ca_ES => "Catalan",
            Locale::de_DE => "German",
            Locale::en_IN => "English (India)",
            Locale::en_US => "English (US)",
            Locale::es_419 | Locale::es_LA => "Spanish (Latin America)",
            Locale::es_ES => "Spanish (European)",
            Locale::fr_FR => "French",
            Locale::hi_IN => "Hindi",
            Locale::id_ID => "Indonesian",
            Locale::it_IT => "Italian",
            Locale::ja_JP => "Japanese",
            Locale::ko_KR => "Korean",
            Locale::ms_MY => "Malay",
            Locale::pl_PL => "Polish",
            Locale::pt_BR => "Portuguese (Brazil)",
            Locale::pt_PT => "Portuguese (Europe)",
            Locale::ru_RU => "Russian",
            Locale::ta_IN => "Tamil",
            Locale::te_IN => "Telugu",
            Locale::th_TH => "Thai",
            Locale::tr_TR => "Turkish",
            Locale::vi_VN => "Vietnamese",
            Locale::zh_CN => "Chinese (China)",
            Locale::zh_HK => "Chinese (Cantonese)",
            Locale::zh_TW => "Chinese (Mandarin)",
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
    pub async fn premium(&self) -> bool {
        self.executor.premium().await
    }

    /// Return the access token used to make requests. The token changes every 5 minutes, so you
    /// might have to re-call this function if you have a long-living session where you need it.
    pub async fn access_token(&self) -> String {
        self.executor.config.read().await.access_token.clone()
    }

    /// Return the current session token. It can be used to log-in later with
    /// [`CrunchyrollBuilder::login_with_refresh_token`] or [`CrunchyrollBuilder::login_with_etp_rt`].
    pub async fn session_token(&self) -> SessionToken {
        self.executor.config.read().await.session_token.clone()
    }

    /// Return the device identifier for the current session.
    pub fn device_identifier(&self) -> Option<DeviceIdentifier> {
        self.executor.details.device_identifier.clone()
    }
}

mod auth {
    use crate::error::{Error, check_request};
    use crate::media::StreamPlatform;
    use crate::{Crunchyroll, Locale, Request, Result};
    use chrono::{DateTime, Duration, Utc};
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use reqwest::{Client, ClientBuilder, IntoUrl, RequestBuilder, header};
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use std::ops::Add;
    use std::sync::Arc;
    use tokio::sync::RwLock;

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
        /// Using [`Uuid::new_v4`] for it works fine.
        pub device_id: String,
        /// Type of the device which issues the session, e.g. `Chrome on Windows` or `iPhone 15`.
        pub device_type: String,
        /// Name of the device which issues the session. This may be empty, for example all session
        /// that are created over the website have an empty name; when issues via the app, the name
        /// is the name of your phone (which you can modify/set when you set up the phone).
        pub device_name: Option<String>,
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
        /// Is [`None`] if generated via [`Executor::auth_anonymously`].
        profile_id: Option<String>,
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
        pub(crate) device_identifier: Option<DeviceIdentifier>,
        pub(crate) stream_platform: StreamPlatform,

        pub(crate) bucket: String,

        pub(crate) signature: String,
        pub(crate) policy: String,
        pub(crate) key_pair_id: String,
        /// The account id is wrapped in a [`Result`] since [`Executor::auth_anonymously`] /
        /// [`CrunchyrollBuilder::login_anonymously`] doesn't return an account id and to prevent
        /// writing error messages multiple times in functions which require the account id to be
        /// set they can just get the id or return the fix set error message.
        pub(crate) account_id: Result<String>,
    }

    #[cfg(feature = "experimental-stabilizations")]
    /// Contains which fixes should be used to make the api more reliable as Crunchyroll does weird
    /// stuff / delivers incorrect results.
    #[derive(Clone, Debug)]
    pub(crate) struct ExecutorFixes {
        pub(crate) locale_name_parsing: bool,
        pub(crate) season_number: bool,
    }

    /// Internal struct to execute all request with.
    #[derive(Debug)]
    pub struct Executor {
        pub(crate) client: Client,

        /// Must be a [`RwLock`] because `Executor` is always passed inside `Arc` which does not
        /// allow direct changes to the struct.
        pub(crate) config: RwLock<ExecutorConfig>,
        pub(crate) details: ExecutorDetails,

        #[cfg(feature = "tower")]
        pub(crate) middleware: Option<tokio::sync::Mutex<crate::internal::tower::Middleware>>,
        #[cfg(feature = "experimental-stabilizations")]
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
            req = self.auth_req(req).await?;
            req = req.header(header::CONTENT_TYPE, "application/json");

            let mut resp: T = request(
                &self.client,
                req,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;

            resp.__set_executor(self.clone()).await;

            Ok(resp)
        }

        pub(crate) async fn auth_req(
            self: &Arc<Self>,
            mut req: RequestBuilder,
        ) -> Result<RequestBuilder> {
            let mut config = self.config.write().await;
            if config.session_expire <= Utc::now() {
                let login_response = match &config.session_token {
                    SessionToken::RefreshToken(refresh_token) => {
                        Executor::auth_with_refresh_token(
                            &self.client,
                            refresh_token.as_str(),
                            &self.details.device_identifier,
                            #[cfg(feature = "tower")]
                            self.middleware.as_ref(),
                        )
                        .await?
                    }
                    SessionToken::EtpRt(etp_rt) => {
                        Executor::auth_with_etp_rt(
                            &self.client,
                            etp_rt.as_str(),
                            &self.details.device_identifier,
                            #[cfg(feature = "tower")]
                            self.middleware.as_ref(),
                        )
                        .await?
                    }
                    SessionToken::Anonymous => {
                        Executor::auth_anonymously(
                            &self.client,
                            &self.details.device_identifier,
                            #[cfg(feature = "tower")]
                            self.middleware.as_ref(),
                        )
                        .await?
                    }
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
                new_config.session_expire = Utc::now()
                    .add(Duration::try_seconds(login_response.expires_in as i64).unwrap());

                *config = new_config;
            }

            req = req.header(
                header::AUTHORIZATION,
                format!("Bearer {}", config.access_token),
            );
            Ok(req)
        }

        pub(crate) async fn jwt_claim<T: DeserializeOwned>(
            &self,
            claim: &str,
        ) -> Result<Option<T>> {
            let executor_config = self.config.read().await;

            let token = executor_config.access_token.as_str();
            let key = jsonwebtoken::DecodingKey::from_rsa_components("", "").unwrap();
            let mut validation = jsonwebtoken::Validation::default();
            // the jwt might be expired when calling this function. but there is no really need to
            // refresh it if this case happens. sure, it might be that something has changed when
            // re-requesting the token but the possibility of this is tiny
            validation.validate_exp = false;
            // we just want the jwt claims, no need to check the signature. no safety critical
            // processes rely on the jwt internally
            validation.insecure_disable_signature_validation();

            let mut claims = jsonwebtoken::decode::<serde_json::Map<String, serde_json::Value>>(
                token,
                &key,
                &validation,
            )
            .unwrap()
            .claims;
            if let Some(claim) = claims.remove(claim) {
                Ok(serde_json::from_value(claim)?)
            } else {
                Ok(None)
            }
        }

        pub(crate) async fn premium(&self) -> bool {
            self.jwt_claim::<Vec<String>>("benefits")
                .await
                .unwrap()
                .unwrap_or_default()
                .contains(&"cr_premium".to_string())
        }

        fn auth_body<'a>(
            mut pre_body: Vec<(&'a str, &'a str)>,
            device_identifier: &'a Option<DeviceIdentifier>,
        ) -> Vec<(&'a str, &'a str)> {
            pre_body.push(("scope", "offline_access"));
            if let Some(device_identifier) = device_identifier {
                pre_body.push(("device_id", device_identifier.device_id.as_str()));
                pre_body.push(("device_type", device_identifier.device_type.as_str()));
                if let Some(device_name) = &device_identifier.device_name {
                    pre_body.push(("device_name", device_name.as_str()));
                }
            }
            pre_body
        }

        async fn auth_anonymously(
            client: &Client,
            device_identifier: &Option<DeviceIdentifier>,
            #[cfg(feature = "tower")] middleware: Option<
                &tokio::sync::Mutex<crate::internal::tower::Middleware>,
            >,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let body = Self::auth_body(vec![("grant_type", "client_id")], device_identifier);
            let req = client
                .post(endpoint)
                .header(header::AUTHORIZATION, "Basic dC1rZGdwMmg4YzNqdWI4Zm4wZnE6eWZMRGZNZnJZdktYaDRKWFMxTEVJMmNDcXUxdjVXYW4=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header("ETP-Anonymous-ID", uuid::Uuid::new_v4().to_string())
                .body(serde_urlencoded::to_string(body).unwrap())
                .build()?;
            #[cfg(not(feature = "tower"))]
            let resp = client.execute(req).await?;
            #[cfg(feature = "tower")]
            let resp = {
                use std::ops::DerefMut;
                if let Some(middleware) = middleware {
                    middleware.lock().await.deref_mut().call(req).await?
                } else {
                    client.execute(req).await?
                }
            };

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_credentials(
            client: &Client,
            email: &str,
            password: &str,
            device_identifier: &Option<DeviceIdentifier>,
            #[cfg(feature = "tower")] middleware: Option<
                &tokio::sync::Mutex<crate::internal::tower::Middleware>,
            >,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let body = Self::auth_body(
                vec![
                    ("username", email),
                    ("password", password),
                    ("grant_type", "password"),
                ],
                device_identifier,
            );
            let req = client.post(endpoint)
                .header(header::AUTHORIZATION, "Basic eHVuaWh2ZWRidDNtYmlzdWhldnQ6MWtJUzVkeVR2akUwX3JxYUEzWWVBaDBiVVhVbXhXMTE=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(serde_urlencoded::to_string(body).unwrap())
                .build()?;
            #[cfg(not(feature = "tower"))]
            let resp = client.execute(req).await?;
            #[cfg(feature = "tower")]
            let resp = {
                use std::ops::DerefMut;
                if let Some(middleware) = middleware {
                    middleware.lock().await.deref_mut().call(req).await?
                } else {
                    client.execute(req).await?
                }
            };

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_refresh_token(
            client: &Client,
            refresh_token: &str,
            device_identifier: &Option<DeviceIdentifier>,
            #[cfg(feature = "tower")] middleware: Option<
                &tokio::sync::Mutex<crate::internal::tower::Middleware>,
            >,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let body = Self::auth_body(
                vec![
                    ("refresh_token", refresh_token),
                    ("grant_type", "refresh_token"),
                ],
                device_identifier,
            );
            let req = client.post(endpoint)
                .header(header::AUTHORIZATION, "Basic eHVuaWh2ZWRidDNtYmlzdWhldnQ6MWtJUzVkeVR2akUwX3JxYUEzWWVBaDBiVVhVbXhXMTE=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(serde_urlencoded::to_string(body).unwrap())
                .build()?;
            #[cfg(not(feature = "tower"))]
            let resp = client.execute(req).await?;
            #[cfg(feature = "tower")]
            let resp = {
                use std::ops::DerefMut;
                if let Some(middleware) = middleware {
                    middleware.lock().await.deref_mut().call(req).await?
                } else {
                    client.execute(req).await?
                }
            };

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_refresh_token_profile_id(
            client: &Client,
            refresh_token: &str,
            profile_id: &str,
            device_identifier: &Option<DeviceIdentifier>,
            #[cfg(feature = "tower")] middleware: Option<
                &tokio::sync::Mutex<crate::internal::tower::Middleware>,
            >,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let body = Self::auth_body(
                vec![
                    ("refresh_token", refresh_token),
                    ("grant_type", "refresh_token_profile_id"),
                    ("profile_id", profile_id),
                ],
                device_identifier,
            );
            let req = client.post(endpoint)
                .header(header::AUTHORIZATION, "Basic dC1rZGdwMmg4YzNqdWI4Zm4wZnE6eWZMRGZNZnJZdktYaDRKWFMxTEVJMmNDcXUxdjVXYW4=")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(serde_urlencoded::to_string(body).unwrap())
                .build()?;
            #[cfg(not(feature = "tower"))]
            let resp = client.execute(req).await?;
            #[cfg(feature = "tower")]
            let resp = {
                use std::ops::DerefMut;
                if let Some(middleware) = middleware {
                    middleware.lock().await.deref_mut().call(req).await?
                } else {
                    client.execute(req).await?
                }
            };

            check_request(endpoint.to_string(), resp).await
        }

        async fn auth_with_etp_rt(
            client: &Client,
            etp_rt: &str,
            device_identifier: &Option<DeviceIdentifier>,
            #[cfg(feature = "tower")] middleware: Option<
                &tokio::sync::Mutex<crate::internal::tower::Middleware>,
            >,
        ) -> Result<AuthResponse> {
            let endpoint = "https://www.crunchyroll.com/auth/v1/token";
            let body = Self::auth_body(vec![("grant_type", "etp_rt_cookie")], device_identifier);
            let req = client
                .post(endpoint)
                .header(header::AUTHORIZATION, "Basic bm9haWhkZXZtXzZpeWcwYThsMHE6")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, format!("etp_rt={etp_rt}"))
                .body(serde_urlencoded::to_string(body).unwrap())
                .build()?;
            #[cfg(not(feature = "tower"))]
            let resp = client.execute(req).await?;
            #[cfg(feature = "tower")]
            let resp = {
                use std::ops::DerefMut;
                if let Some(middleware) = middleware {
                    middleware.lock().await.deref_mut().call(req).await?
                } else {
                    client.execute(req).await?
                }
            };

            check_request(endpoint.to_string(), resp).await
        }
    }

    impl Default for Executor {
        fn default() -> Self {
            Self {
                client: Client::new(),
                config: RwLock::new(ExecutorConfig {
                    token_type: "".to_string(),
                    access_token: "".to_string(),
                    session_token: SessionToken::RefreshToken("".into()),
                    session_expire: Default::default(),
                }),
                details: ExecutorDetails {
                    locale: Default::default(),
                    preferred_audio_locale: None,
                    device_identifier: None,
                    stream_platform: Default::default(),
                    bucket: "".to_string(),
                    signature: "".to_string(),
                    policy: "".to_string(),
                    key_pair_id: "".to_string(),
                    account_id: Ok("".to_string()),
                },
                #[cfg(feature = "tower")]
                middleware: None,
                #[cfg(feature = "experimental-stabilizations")]
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

        pub(crate) async fn request_raw(mut self, auth: bool) -> Result<Vec<u8>> {
            if auth {
                self.builder = self.executor.auth_req(self.builder).await?;
            }

            #[cfg(feature = "tower")]
            if let Some(middleware) = &self.executor.middleware {
                return Ok(middleware
                    .lock()
                    .await
                    .call(self.builder.build()?)
                    .await?
                    .bytes()
                    .await?
                    .to_vec());
            }
            Ok(self.builder.send().await?.bytes().await?.to_vec())
        }
    }

    /// A builder to construct a new [`Crunchyroll`] instance. To create it, call
    /// [`Crunchyroll::builder`].
    pub struct CrunchyrollBuilder {
        client: Client,
        locale: Locale,
        preferred_audio_locale: Option<Locale>,
        device_identifier: Option<DeviceIdentifier>,
        stream_platform: StreamPlatform,

        #[cfg(feature = "tower")]
        middleware: Option<tokio::sync::Mutex<crate::internal::tower::Middleware>>,
        #[cfg(feature = "experimental-stabilizations")]
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
                device_identifier: None,
                stream_platform: StreamPlatform::default(),
                #[cfg(feature = "tower")]
                middleware: None,
                #[cfg(feature = "experimental-stabilizations")]
                fixes: ExecutorFixes {
                    locale_name_parsing: false,
                    season_number: false,
                },
            }
        }
    }

    impl CrunchyrollBuilder {
        pub const DEFAULT_HEADERS: [(HeaderName, HeaderValue); 4] = [
            (
                header::USER_AGENT,
                HeaderValue::from_static("Crunchyroll/1.8.0 Nintendo Switch/12.3.12.0 UE4/4.27"),
            ),
            (header::ACCEPT, HeaderValue::from_static("*/*")),
            (
                header::ACCEPT_LANGUAGE,
                HeaderValue::from_static("en-US,en;q=0.5"),
            ),
            (header::CONNECTION, HeaderValue::from_static("keep-alive")),
        ];

        /// Return a [`ClientBuilder`] which has all required configurations necessary to send
        /// successful requests to Crunchyroll, applied (most of the time; sometimes Crunchyroll has
        /// fluctuations that requests doesn't work for a specific amount of time and after that
        /// amount everything goes back to normal and works as it should). You can use this builder
        /// to configure the behavior of the download client. Use [`CrunchyrollBuilder::client`] or
        /// to set your built client.
        pub fn predefined_client_builder() -> ClientBuilder {
            let tls_config = rustls::ClientConfig::builder_with_provider(
                rustls::crypto::CryptoProvider {
                    cipher_suites: rustls::crypto::ring::DEFAULT_CIPHER_SUITES.to_vec(),
                    kx_groups: vec![rustls::crypto::ring::kx_group::X25519],
                    ..rustls::crypto::ring::default_provider()
                }
                .into(),
            )
            .with_protocol_versions(&[&rustls::version::TLS12, &rustls::version::TLS13])
            .unwrap()
            .with_root_certificates(rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            })
            .with_no_client_auth();

            Client::builder()
                .https_only(true)
                .cookie_store(true)
                .default_headers(HeaderMap::from_iter(CrunchyrollBuilder::DEFAULT_HEADERS))
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

        /// Set an identifier for the session which will be opened. `device_id` is usually a random
        /// UUID, `device_type` a description of the device which issues the session, e.g. `Chrome
        /// on Windows` or `iPhone 15`.
        /// Gets only used if the login method is [`CrunchyrollBuilder::login_with_credentials`].
        pub fn device_identifier(
            mut self,
            device_identifier: DeviceIdentifier,
        ) -> CrunchyrollBuilder {
            self.device_identifier = Some(device_identifier);
            self
        }

        pub fn stream_platform(mut self, stream_platform: StreamPlatform) -> CrunchyrollBuilder {
            self.stream_platform = stream_platform;
            self
        }

        /// Adds a [tower](https://docs.rs/tower/latest/tower/) middleware which is called on every
        /// request.
        #[cfg(feature = "tower")]
        #[cfg_attr(docsrs, doc(cfg(feature = "tower")))]
        pub fn middleware<F, S>(mut self, service: S) -> CrunchyrollBuilder
        where
            F: std::future::Future<Output = Result<reqwest::Response, Error>> + Send + 'static,
            S: tower_service::Service<
                    reqwest::Request,
                    Response = reqwest::Response,
                    Error = Error,
                    Future = F,
                > + Send
                + 'static,
        {
            self.middleware = Some(tokio::sync::Mutex::new(
                crate::internal::tower::Middleware::new(service),
            ));
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
            self.pre_login().await?;

            let login_response = Executor::auth_anonymously(
                &self.client,
                &self.device_identifier,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;
            let session_token = SessionToken::Anonymous;

            self.post_login(login_response, session_token).await
        }

        /// Logs in with credentials (email and password) and returns a new `Crunchyroll` instance.
        pub async fn login_with_credentials<S: AsRef<str>>(
            self,
            email: S,
            password: S,
        ) -> Result<Crunchyroll> {
            self.pre_login().await?;

            let login_response = Executor::auth_with_credentials(
                &self.client,
                email.as_ref(),
                password.as_ref(),
                &self.device_identifier,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
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
            self.pre_login().await?;

            let login_response = Executor::auth_with_refresh_token(
                &self.client,
                refresh_token.as_ref(),
                &self.device_identifier,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;
            let session_token =
                SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

            self.post_login(login_response, session_token).await
        }

        /// Just like [`CrunchyrollBuilder::login_with_refresh_token`] but with the addition that
        /// the id of a [`crate::profile::Profile`] is given too. The resulting [`Crunchyroll`]
        /// session will settings that are specific to the given [`crate::profile::Profile`] id.
        ///
        /// *Note*: When using this login method, some endpoints aren't available / will return an
        /// error. Idk why, but these endpoints can only be used if the authentication is anything
        /// other than [`CrunchyrollBuilder::login_with_refresh_token_profile_id`].
        pub async fn login_with_refresh_token_profile_id<S: AsRef<str>>(
            self,
            refresh_token: S,
            profile_id: S,
        ) -> Result<Crunchyroll> {
            self.pre_login().await?;

            let login_response = Executor::auth_with_refresh_token_profile_id(
                &self.client,
                refresh_token.as_ref(),
                profile_id.as_ref(),
                &self.device_identifier,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;
            let session_token =
                SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

            self.post_login(login_response, session_token).await
        }

        /// Logs in with the `etp_rt` cookie that is generated when logging in with the browser and
        /// returns a new `Crunchyroll` instance. This cookie can be extracted if you copy the
        /// `etp_rt` cookie from your browser.
        /// *Note*: You need to set [`CrunchyrollBuilder::device_identifier`] to the same identifier
        /// which were used in the login that initially created the `etp_rt` cookie, otherwise the
        /// login will fail.
        pub async fn login_with_etp_rt<S: AsRef<str>>(self, etp_rt: S) -> Result<Crunchyroll> {
            self.pre_login().await?;

            let login_response = Executor::auth_with_etp_rt(
                &self.client,
                etp_rt.as_ref(),
                &self.device_identifier,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;
            let session_token = SessionToken::EtpRt(login_response.refresh_token.clone().unwrap());

            self.post_login(login_response, session_token).await
        }

        async fn pre_login(&self) -> Result<()> {
            // Request the index page to set cookies which are required to bypass the cloudflare bot
            // check
            self.client
                .get("https://www.crunchyroll.com")
                .send()
                .await?;
            Ok(())
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
            let index: IndexResp = request(
                &self.client,
                index_req,
                #[cfg(feature = "tower")]
                self.middleware.as_ref(),
            )
            .await?;

            let crunchy = Crunchyroll {
                executor: Arc::new(Executor {
                    client: self.client,

                    config: RwLock::new(ExecutorConfig {
                        token_type: login_response.token_type,
                        access_token: login_response.access_token,
                        session_token,
                        session_expire: Utc::now()
                            .add(Duration::try_seconds(login_response.expires_in as i64).unwrap()),
                    }),
                    details: ExecutorDetails {
                        locale: self.locale,
                        preferred_audio_locale: self.preferred_audio_locale,
                        device_identifier: self.device_identifier,
                        stream_platform: self.stream_platform,

                        // '/' is trimmed so that urls which require it must be in .../{bucket}/... like format.
                        // this just looks cleaner
                        bucket: index
                            .cms_web
                            .bucket
                            .strip_prefix('/')
                            .unwrap_or(index.cms_web.bucket.as_str())
                            .to_string(),

                        signature: index.cms_web.signature,
                        policy: index.cms_web.policy,
                        key_pair_id: index.cms_web.key_pair_id,
                        account_id: login_response.account_id.ok_or_else(|| {
                            Error::Authentication {
                                message: "Login with a user account to use this function"
                                    .to_string(),
                            }
                        }),
                    },
                    #[cfg(feature = "tower")]
                    middleware: self.middleware,
                    #[cfg(feature = "experimental-stabilizations")]
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
        #[cfg(feature = "tower")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::tower::Middleware>,
        >,
    ) -> Result<T> {
        let built_req = req.build()?;
        let url = built_req.url().to_string();
        #[cfg(not(feature = "tower"))]
        let resp = client.execute(built_req).await?;
        #[cfg(feature = "tower")]
        let resp = {
            use std::ops::DerefMut;
            if let Some(middleware) = middleware {
                middleware.lock().await.deref_mut().call(built_req).await?
            } else {
                client.execute(built_req).await?
            }
        };

        #[cfg(not(feature = "__test_strict"))]
        {
            check_request(url, resp).await
        }
        #[cfg(feature = "__test_strict")]
        {
            let result = check_request(url.clone(), resp).await?;

            let cleaned = clean_request(result);
            let value = serde_json::Value::deserialize(serde::de::value::MapDeserializer::new(
                cleaned.into_iter(),
            ))?;
            serde_json::from_value(value.clone()).map_err(|e| Error::Decode {
                message: format!("{} at {}:{}", e, e.line(), e.column()),
                content: value.to_string().into_bytes(),
                url,
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
pub use auth::{CrunchyrollBuilder, DeviceIdentifier, SessionToken};
