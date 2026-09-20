//! Builder and access to the [`Crunchyroll`] struct which is required to make any action.

use crate::{
    Executor,
    auth::{CrunchyrollBuilder, DeviceIdentifier, DevicePlatform, SessionToken},
};

use reqwest::Client;
use std::sync::Arc;

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
        self.executor.session.read().await.access_token.clone()
    }

    /// Return the current session token. It can be used to log-in later with
    /// [`CrunchyrollBuilder::login_with_refresh_token`] or [`CrunchyrollBuilder::login_with_etp_rt`].
    pub async fn session_token(&self) -> SessionToken {
        self.executor.session.read().await.session_token.clone()
    }

    /// Return the device identifier for the current session.
    pub fn device_identifier(&self) -> DeviceIdentifier {
        self.executor.details.device_identifier.clone()
    }

    /// Return the platform for the current session. This is the platform that was
    /// configured via [`CrunchyrollBuilder::platform`].
    pub fn device_platform(&self) -> DevicePlatform {
        self.executor.details.device_platform.clone()
    }
}
