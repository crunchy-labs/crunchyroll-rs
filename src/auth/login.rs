use super::DeviceIdentifier;
use super::executor::Executor;
use crate::error::{Result, check_request};
use reqwest::{Client, header};
use serde::Deserialize;

/// Information about the login response of the Crunchyroll api.
#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "__test_strict", serde(deny_unknown_fields))]
#[cfg_attr(not(feature = "__test_strict"), serde(default))]
#[allow(dead_code)]
pub(super) struct AuthResponse {
    pub(super) access_token: String,
    /// Is [`None`] if generated via [`Executor::auth_anonymously`].
    pub(super) refresh_token: Option<String>,
    pub(super) expires_in: i32,
    pub(super) token_type: String,
    pub(super) scope: String,
    pub(super) country: String,
    /// Is [`None`] if generated via [`Executor::auth_anonymously`].
    pub(super) account_id: Option<String>,
    /// Is [`None`] if generated via [`Executor::auth_anonymously`].
    pub(super) profile_id: Option<String>,
}

impl Executor {
    pub(super) fn auth_body<'a>(
        mut pre_body: Vec<(&'a str, &'a str)>,
        device_identifier: &'a DeviceIdentifier,
    ) -> Vec<(&'a str, &'a str)> {
        pre_body.push(("scope", "offline_access"));
        pre_body.push(("device_id", device_identifier.device_id.as_str()));
        pre_body.push(("device_type", device_identifier.device_type.as_str()));
        if let Some(device_name) = &device_identifier.device_name {
            pre_body.push(("device_name", device_name.as_str()));
        }
        pre_body
    }

    pub(super) async fn send_auth_req(
        client: &Client,
        req: reqwest::Request,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
        >,
    ) -> Result<AuthResponse> {
        #[cfg(not(feature = "middleware"))]
        let resp = client.execute(req).await?;
        #[cfg(feature = "middleware")]
        let resp = {
            use std::ops::DerefMut;
            if let Some(middleware) = middleware {
                let url = req.url().to_string();
                middleware
                    .lock()
                    .await
                    .deref_mut()
                    .call(crate::middleware::MiddlewareContext::new(client, req))
                    .await
                    .map_err(|e| crate::internal::middleware::middleware_error_to_error(e, url))?
            } else {
                client.execute(req).await?
            }
        };

        check_request(resp).await
    }

    pub(super) async fn auth_anonymously(
        client: &Client,
        device_identifier: &DeviceIdentifier,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
        >,
    ) -> Result<AuthResponse> {
        let endpoint = "https://www.crunchyroll.com/auth/v1/token";
        let body = Self::auth_body(vec![("grant_type", "client_id")], device_identifier);
        let req = client
            .post(endpoint)
            .header(
                header::AUTHORIZATION,
                "Basic dC1rZGdwMmg4YzNqdWI4Zm4wZnE6eWZMRGZNZnJZdktYaDRKWFMxTEVJMmNDcXUxdjVXYW4=",
            )
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("ETP-Anonymous-ID", &device_identifier.device_id)
            .body(serde_urlencoded::to_string(body).unwrap())
            .build()?;

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }

    pub(super) async fn auth_with_credentials(
        client: &Client,
        email: &str,
        password: &str,
        device_identifier: &DeviceIdentifier,
        basic_auth_token: &str,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
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
        let req = client
            .post(endpoint)
            .header(header::AUTHORIZATION, format!("Basic {basic_auth_token}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header("ETP-Anonymous-ID", &device_identifier.device_id)
            .body(serde_urlencoded::to_string(body).unwrap())
            .build()?;

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }

    pub(super) async fn auth_with_refresh_token(
        client: &Client,
        refresh_token: &str,
        device_identifier: &DeviceIdentifier,
        basic_auth_token: &str,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
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
        let req = client
            .post(endpoint)
            .header(header::AUTHORIZATION, format!("Basic {basic_auth_token}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(body).unwrap())
            .build()?;

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }

    pub(super) async fn auth_with_refresh_token_profile_id(
        client: &Client,
        refresh_token: &str,
        profile_id: &str,
        device_identifier: &DeviceIdentifier,
        basic_auth_token: &str,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
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
        let req = client
            .post(endpoint)
            .header(header::AUTHORIZATION, format!("Basic {basic_auth_token}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(body).unwrap())
            .build()?;

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }

    pub(super) async fn auth_with_etp_rt(
        client: &Client,
        etp_rt: &str,
        device_identifier: &DeviceIdentifier,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
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

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }

    pub(super) async fn auth_with_oauth_code(
        client: &Client,
        code: &str,
        code_verifier: &str,
        device_identifier: &DeviceIdentifier,
        basic_auth_token: &str,
        #[cfg(feature = "middleware")] middleware: Option<
            &tokio::sync::Mutex<crate::internal::middleware::Middleware>,
        >,
    ) -> Result<AuthResponse> {
        let endpoint = "https://www.crunchyroll.com/auth/v1/token";
        let body = Self::auth_body(
            vec![
                ("code", code),
                ("code_verifier", code_verifier),
                ("grant_type", "authorization_code"),
            ],
            device_identifier,
        );
        let req = client
            .post(endpoint)
            .header(header::AUTHORIZATION, format!("Basic {basic_auth_token}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(body).unwrap())
            .build()?;

        Self::send_auth_req(
            client,
            req,
            #[cfg(feature = "middleware")]
            middleware,
        )
        .await
    }
}
