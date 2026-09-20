use super::executor::{Executor, ExecutorDetails, ExecutorSession};
use super::login::AuthResponse;
use super::{DeviceIdentifier, DevicePlatform, SessionToken};
use crate::Crunchyroll;
use crate::Locale;
use crate::auth::platform_credentials;
use crate::error::{Error, Result};
use chrono::{Duration, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, ClientBuilder, header};
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::RwLock;

struct ResolvedCrunchyrollBuilder {
    client: Client,
    locale: Locale,
    preferred_audio_locale: Option<Locale>,

    device_platform: DevicePlatform,
    basic_auth_token: String,

    #[cfg(feature = "middleware")]
    middleware: Option<tokio::sync::Mutex<crate::internal::middleware::Middleware>>,
    #[cfg(feature = "experimental-stabilizations")]
    fixes: super::executor::ExecutorFixes,
}

impl ResolvedCrunchyrollBuilder {
    fn build(
        self,
        login_response: AuthResponse,
        session_token: SessionToken,
        device_identifier: DeviceIdentifier,
    ) -> Crunchyroll {
        Crunchyroll {
            executor: Arc::new(Executor {
                client: self.client,
                session: RwLock::new(ExecutorSession {
                    token_type: login_response.token_type,
                    access_token: login_response.access_token,
                    session_token,
                    session_expire: Utc::now()
                        .add(Duration::try_seconds(login_response.expires_in as i64).unwrap()),
                }),
                details: ExecutorDetails {
                    locale: self.locale,
                    preferred_audio_locale: self.preferred_audio_locale,
                    device_identifier,
                    device_platform: self.device_platform,
                    basic_auth_token: self.basic_auth_token,

                    account_id: login_response.account_id,
                },
                #[cfg(feature = "middleware")]
                middleware: self.middleware,
                #[cfg(feature = "experimental-stabilizations")]
                fixes: self.fixes,
            }),
        }
    }
}

struct CrunchyrollBuilderSessionDetails {
    device_platform: DevicePlatform,
    basic_auth_token: String,
    user_agent: Option<String>,
}

/// A builder to construct a new [`Crunchyroll`] instance. To create it, call
/// [`Crunchyroll::builder`].
pub struct CrunchyrollBuilder {
    client: Option<Client>,
    locale: Locale,
    preferred_audio_locale: Option<Locale>,
    session_details: Option<CrunchyrollBuilderSessionDetails>,

    #[cfg(feature = "middleware")]
    middleware: Option<tokio::sync::Mutex<crate::internal::middleware::Middleware>>,
    #[cfg(feature = "experimental-stabilizations")]
    fixes: super::executor::ExecutorFixes,
}

impl Default for CrunchyrollBuilder {
    fn default() -> Self {
        Self {
            client: None,
            locale: Locale::en_US,
            preferred_audio_locale: None,
            session_details: None,
            #[cfg(feature = "middleware")]
            middleware: None,
            #[cfg(feature = "experimental-stabilizations")]
            fixes: super::executor::ExecutorFixes {
                locale_name_parsing: false,
                season_number: false,
            },
        }
    }
}

impl CrunchyrollBuilder {
    pub const ANDROID_TV_DEFAULT_HEADERS: [(HeaderName, HeaderValue); 3] = [
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
            .default_headers(HeaderMap::from_iter(
                CrunchyrollBuilder::ANDROID_TV_DEFAULT_HEADERS,
            ))
            .use_preconfigured_tls(tls_config)
    }

    /// Set a custom client that will be used in all api requests.
    /// It is recommended to use the client builder from
    /// [`CrunchyrollBuilder::predefined_client_builder`] as base as it has some configurations
    /// which may be needed to make successful requests to Crunchyroll.
    pub fn client(mut self, client: Client) -> CrunchyrollBuilder {
        self.client = Some(client);
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
    pub fn preferred_audio_locale(mut self, preferred_audio_locale: Locale) -> CrunchyrollBuilder {
        self.preferred_audio_locale = Some(preferred_audio_locale);
        self
    }

    /// Sets the platform for which a session should be issued for.
    ///
    /// The three arguments belong together: a basic auth token is only valid for the platform
    /// it was issued for and the user agent must almost always represent the platform too.
    /// For example, the basic auth token bundled with the Android phone app is only valid for
    /// [`DevicePlatform::AndroidPhone`]; using it with any other platform
    /// (e.g. [`DevicePlatform::TvAndroid`]) will cause stream requests to fail with an error.
    ///
    /// The user agent should match the stream platform as well, otherwise requests may fail. You
    /// can pass the user agent here, but it's only set if you don't override the client via
    /// [`CrunchyrollBuilder::client`]. Either way you should set an user agent, the request will
    /// fail otherwise.
    ///
    /// Crunchyroll rotates the basic auth tokens from time to time, which would result in
    /// failed logins if those auth tokens aren't also changed in this crate. To prevent this
    /// issue, the auth token and user agent are fetched dynamically from
    /// [crunchy-labs/artifacts](https://github.com/crunchy-labs/artifacts).
    /// This happens every time you login. It's strongly advised that you implement the fetching
    /// process yourself, and use some sort of caching. You can use
    /// [`platform_credentials::get_platform_credentials`] to get the credentials from the
    /// crunchy-labs/artifacts GitHub repo, or implement it completely yourself.
    ///
    /// Not every login method is available with every basic auth token. For example, the
    /// Android phone basic auth token only supports
    /// [`CrunchyrollBuilder::login_with_oauth_code`] and the refresh token based methods;
    /// [`CrunchyrollBuilder::login_with_credentials`] will be rejected.
    pub fn platform(
        mut self,
        device_platform: DevicePlatform,
        basic_auth_token: String,
        user_agent: Option<String>,
    ) -> CrunchyrollBuilder {
        self.session_details = Some(CrunchyrollBuilderSessionDetails {
            device_platform,
            basic_auth_token,
            user_agent,
        });
        self
    }

    /// Adds a [tower-service](https://docs.rs/tower-service) middleware which is called on
    /// every request.
    #[cfg(feature = "middleware")]
    #[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
    pub fn middleware<E, F, S>(mut self, service: S) -> CrunchyrollBuilder
    where
        E: Into<Box<dyn std::error::Error + Send + Sync + 'static>> + 'static,
        F: Future<Output = Result<reqwest::Response, E>> + Send + 'static,
        S: for<'a> tower_service::Service<
                crate::auth::middleware::MiddlewareContext<'a>,
                Response = reqwest::Response,
                Error = E,
                Future = F,
            > + Send
            + 'static,
    {
        self.middleware = Some(tokio::sync::Mutex::new(
            crate::internal::middleware::Middleware::new(service),
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
    ///
    /// This method won't respect [`CrunchyrollBuilder::platform`] as anonymous login is only
    /// available on the web platform and thus, the credentials are hard-coded.
    pub async fn login_anonymously(
        self,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_anonymously(
            &resolved.client,
            &device_identifier,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token = SessionToken::Anonymous;

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    /// Logs in with credentials (email and password) and returns a new [`Crunchyroll`] instance.
    ///
    /// *Note*: All logins you do with the generated refresh token must have the same
    /// `device_identifier`, otherwise the login will fail.
    ///
    /// *Note*: Many platforms aren't supporting this login method. Expect login errors when
    /// using a custom [`CrunchyrollBuilder::platform`]. Consider using
    /// [`CrunchyrollBuilder::login_with_oauth_code`] instead.
    pub async fn login_with_credentials<S: AsRef<str>>(
        self,
        email: S,
        password: S,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_with_credentials(
            &resolved.client,
            email.as_ref(),
            password.as_ref(),
            &device_identifier,
            &resolved.basic_auth_token,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token =
            SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    /// Logs in with a refresh token. This token is obtained when logging in with
    /// [`CrunchyrollBuilder::login_with_credentials`].
    ///
    /// *Note*: Even though the tokens used in [`CrunchyrollBuilder::login_with_refresh_token`]
    /// and [`CrunchyrollBuilder::login_with_etp_rt`] are having the same syntax, Crunchyroll
    /// internal they're different. I had issues when I tried to log in with the refresh token
    /// on [`CrunchyrollBuilder::login_with_etp_rt`] and vice versa.
    ///
    /// *Note*: You need to set the `device_identifier` to the same identifier which were used
    /// in the login that initially created the refresh token, otherwise the login will fail.
    pub async fn login_with_refresh_token<S: AsRef<str>>(
        self,
        refresh_token: S,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_with_refresh_token(
            &resolved.client,
            refresh_token.as_ref(),
            &device_identifier,
            &resolved.basic_auth_token,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token =
            SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    /// Just like [`CrunchyrollBuilder::login_with_refresh_token`] but with the addition that
    /// the id of a [`crate::profile::Profile`] is given too. The resulting [`Crunchyroll`]
    /// session will settings that are specific to the given [`crate::profile::Profile`] id.
    ///
    /// *Note*: When using this login method, some endpoints aren't available / will return an
    /// error. Idk why, but these endpoints can only be used if the authentication is anything
    /// other than [`CrunchyrollBuilder::login_with_refresh_token_profile_id`].
    ///
    /// *Note*: You need to set the `device_identifier` to the same identifier which were used
    /// in the login that initially created the refresh token, otherwise the login will fail.
    pub async fn login_with_refresh_token_profile_id<S: AsRef<str>>(
        self,
        refresh_token: S,
        profile_id: S,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_with_refresh_token_profile_id(
            &resolved.client,
            refresh_token.as_ref(),
            profile_id.as_ref(),
            &device_identifier,
            &resolved.basic_auth_token,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token =
            SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    /// Logs in with the `etp_rt` cookie that is generated when logging in with the browser and
    /// returns a new [`Crunchyroll`] instance. This cookie can be extracted if you copy the
    /// `etp_rt` cookie from your browser.
    ///
    /// This method uses a hardcoded basic auth token which is independent of the one set
    /// via [`CrunchyrollBuilder::platform`]. The [`DevicePlatform`] configured via
    /// [`CrunchyrollBuilder::platform`] does still apply.
    ///
    /// *Note*: You need to set the `device_identifier` to the same identifier which were used
    /// in the login that initially created the `etp_rt` cookie, otherwise the login will fail.
    pub async fn login_with_etp_rt<S: AsRef<str>>(
        self,
        etp_rt: S,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_with_etp_rt(
            &resolved.client,
            etp_rt.as_ref(),
            &device_identifier,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token = SessionToken::EtpRt(login_response.refresh_token.clone().unwrap());

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    /// Log in with an OAuth authorization code and matching `code_verifier`, as returned by
    /// Crunchyroll's SSO endpoint.
    ///
    /// See the `sso-login` directory in the example folder for a complete example that performs
    /// the full OAuth flow via an embedded webview.
    pub async fn login_with_oauth_code<S: AsRef<str>>(
        self,
        code: S,
        code_verifier: S,
        device_identifier: DeviceIdentifier,
    ) -> Result<Crunchyroll> {
        let resolved = self.resolve().await?;

        let login_response = Executor::auth_with_oauth_code(
            &resolved.client,
            code.as_ref(),
            code_verifier.as_ref(),
            &device_identifier,
            &resolved.basic_auth_token,
            #[cfg(feature = "middleware")]
            resolved.middleware.as_ref(),
        )
        .await?;
        let session_token =
            SessionToken::RefreshToken(login_response.refresh_token.clone().unwrap());

        Ok(resolved.build(login_response, session_token, device_identifier))
    }

    async fn resolve(self) -> Result<ResolvedCrunchyrollBuilder> {
        let session_details = match self.session_details {
            Some(session_details) => session_details,
            // fetch default credentials and set them
            None => {
                let platform_credentials = match platform_credentials::get_platform_credentials()
                    .await
                {
                    Ok(platform_credentials) => platform_credentials,
                    Err(e) => {
                        return Err(Error::from(e).update_msg(|err| {
                            let message = format!("Error while fetching app credentials. This is most likely a GitHub issue. Check if GitHub is down and/or {} is available. You may use `CrunchyrollBuilder::platform` to override the credentials", platform_credentials::PLATFORM_CREDENTIALS_URL);
                            Some(match err {
                                Some(msg) => format!("{msg}: {message}"),
                                None => message,
                            })
                        }));
                    }
                };

                CrunchyrollBuilderSessionDetails {
                    device_platform: DevicePlatform::TvAndroid,
                    user_agent: Some(platform_credentials.android_tv_user_agent()),
                    basic_auth_token: platform_credentials.android_tv.basic_auth_token,
                }
            }
        };

        let client = match self.client {
            Some(client) => client,
            None => {
                let mut client_builder = Self::predefined_client_builder();
                if let Some(user_agent) = session_details.user_agent {
                    client_builder = client_builder.user_agent(user_agent.clone())
                }
                client_builder.build().unwrap()
            }
        };

        // Request the index page to set cookies which are required to bypass the cloudflare bot
        // check
        client.get("https://www.crunchyroll.com").send().await?;

        Ok(ResolvedCrunchyrollBuilder {
            client,
            locale: self.locale,
            preferred_audio_locale: self.preferred_audio_locale,
            device_platform: session_details.device_platform,
            basic_auth_token: session_details.basic_auth_token,
            #[cfg(feature = "middleware")]
            middleware: self.middleware,
            #[cfg(feature = "experimental-stabilizations")]
            fixes: self.fixes,
        })
    }
}
