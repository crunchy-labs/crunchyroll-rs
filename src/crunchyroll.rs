use crate::auth::{CrunchyrollBuilder, Executor, SessionToken};
use crate::enum_values;
use crate::error::Result;
use std::sync::Arc;

enum_values! {
    Locale,
    #[derive(Clone, Debug, Hash, Eq, PartialEq)],
    ar_ME = "ar-ME",
    ar_SA = "ar-SA",
    de_DE = "de-DE",
    en_US = "en-US",
    es_419 = "es-419",
    es_ES = "es-ES",
    es_LA = "es-LA",
    fr_FR = "fr-FR",
    it_IT = "it-IT",
    ja_JP = "ja-JP",
    pt_BR = "pt-BR",
    ru_RU = "ru-RU"
}

enum_values! {
    MaturityRating,
    #[derive(Clone, Debug)],
    NotMature = "M2",
    Mature = "M3"
}

/// Starting point of this whole library.
#[derive(Debug, Clone)]
pub struct Crunchyroll {
    pub(crate) executor: Arc<Executor>,
}

/// This impl is only for the native login methods. Compiling to with wasm fails if every function
/// is in here because it don't know how to behave with `reqwest::Client`.
impl Crunchyroll {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> CrunchyrollBuilder {
        CrunchyrollBuilder {
            client: reqwest::Client::new(),
            locale: Locale::en_US,
        }
    }

    pub fn premium(&self) -> bool {
        self.executor.details.premium
    }

    pub async fn session_token(&self) -> SessionToken {
        self.executor.config.lock().await.session_token.clone()
    }

    pub async fn invalidate_session(self) -> Result<()> {
        let endpoint = "https://crunchyroll.com/logout";
        self.executor
            .to_owned()
            .request(self.executor.client.get(endpoint))
            .await
    }
}
