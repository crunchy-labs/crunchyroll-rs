use super::request::{ExecutorRequestBuilder, request};
use super::{DeviceIdentifier, DevicePlatform, SessionToken};
use crate::Locale;
use crate::Request;
use crate::error::{Error, ErrorKind, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::header;
use reqwest::{Client, IntoUrl, RequestBuilder};
use serde::de::DeserializeOwned;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub(crate) struct ExecutorSession {
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
    pub(crate) device_identifier: DeviceIdentifier,
    pub(crate) device_platform: DevicePlatform,
    pub(crate) basic_auth_token: String,

    /// The account id is wrapped in a [`Option`] since [`Executor::auth_anonymously`] /
    /// [`CrunchyrollBuilder::login_anonymously`] doesn't return an account id and to prevent
    /// writing error messages multiple times in functions which require the account id to be
    /// set they can just get the id or return the fix set error message.
    pub(crate) account_id: Option<String>,
}

impl ExecutorDetails {
    pub(crate) fn account_id(&self) -> Result<String> {
        self.account_id.as_ref().cloned().ok_or_else(|| {
            Error::error_from_kind(
                ErrorKind::Input,
                "Login with a user account to use this function",
            )
        })
    }
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

    /// Must be a [`RwLock`] because `Executor` is always passed inside [`Arc`] which does not
    /// allow direct changes to the struct.
    pub(crate) session: RwLock<ExecutorSession>,

    pub(crate) details: ExecutorDetails,

    #[cfg(feature = "middleware")]
    pub(crate) middleware: Option<tokio::sync::Mutex<crate::internal::middleware::Middleware>>,
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
            #[cfg(feature = "middleware")]
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
        let mut session = self.session.write().await;
        if session.session_expire <= Utc::now() {
            let login_response = match &session.session_token {
                SessionToken::RefreshToken(refresh_token) => {
                    Executor::auth_with_refresh_token(
                        &self.client,
                        refresh_token.as_str(),
                        &self.details.device_identifier,
                        &self.details.basic_auth_token,
                        #[cfg(feature = "middleware")]
                        self.middleware.as_ref(),
                    )
                    .await?
                }
                SessionToken::EtpRt(etp_rt) => {
                    Executor::auth_with_etp_rt(
                        &self.client,
                        etp_rt.as_str(),
                        &self.details.device_identifier,
                        #[cfg(feature = "middleware")]
                        self.middleware.as_ref(),
                    )
                    .await?
                }
                SessionToken::Anonymous => {
                    Executor::auth_anonymously(
                        &self.client,
                        &self.details.device_identifier,
                        #[cfg(feature = "middleware")]
                        self.middleware.as_ref(),
                    )
                    .await?
                }
            };

            *session = ExecutorSession {
                token_type: login_response.token_type,
                access_token: login_response.access_token,
                session_token: match session.session_token {
                    SessionToken::RefreshToken(_) => {
                        SessionToken::RefreshToken(login_response.refresh_token.unwrap())
                    }
                    SessionToken::EtpRt(_) => {
                        SessionToken::EtpRt(login_response.refresh_token.unwrap())
                    }
                    SessionToken::Anonymous => SessionToken::Anonymous,
                },
                session_expire: Utc::now()
                    .add(Duration::try_seconds(login_response.expires_in as i64).unwrap()),
            };
        }

        req = req.header(
            header::AUTHORIZATION,
            format!("{} {}", session.token_type, session.access_token),
        );
        Ok(req)
    }

    pub(crate) async fn jwt_claim<T: DeserializeOwned>(&self, claim: &str) -> Result<Option<T>> {
        let executor_session = self.session.read().await;

        let token = executor_session.access_token.as_str();
        // we just want the jwt claims, no need to check the signature. no safety critical
        // processes rely on the jwt internally
        let mut claims = jsonwebtoken::dangerous::insecure_decode::<
            serde_json::Map<String, serde_json::Value>,
        >(token)
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
}

impl Default for Executor {
    fn default() -> Self {
        Self {
            client: Client::new(),
            session: RwLock::new(ExecutorSession {
                token_type: "".to_string(),
                access_token: "".to_string(),
                session_token: SessionToken::RefreshToken("".into()),
                session_expire: Default::default(),
            }),
            details: ExecutorDetails {
                locale: Default::default(),
                preferred_audio_locale: None,
                device_identifier: DeviceIdentifier::default(),
                device_platform: Default::default(),
                basic_auth_token: "".to_string(),
                account_id: None,
            },
            #[cfg(feature = "middleware")]
            middleware: None,
            #[cfg(feature = "experimental-stabilizations")]
            fixes: ExecutorFixes {
                locale_name_parsing: false,
                season_number: false,
            },
        }
    }
}
