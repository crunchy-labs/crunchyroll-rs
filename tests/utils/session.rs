#![allow(dead_code)]

use crate::utils::store::{get_store, has_store, set_store, Store};
use crunchyroll::crunchyroll::SessionToken;
use crunchyroll::Crunchyroll;

pub static SESSION: Store<Crunchyroll> = Store::new(|| {
    Box::pin(async {
        let raw_session = get_store("session".into()).unwrap();
        let (token_type, token) = raw_session.split_once(':').unwrap_or(("", ""));
        let crunchy = match token_type {
            "refresh_token" => Crunchyroll::builder()
                .login_with_refresh_token(token)
                .await
                .unwrap(),
            "etp_rt" => Crunchyroll::builder()
                .login_with_etp_rt(token)
                .await
                .unwrap(),
            _ => panic!("invalid session '{}'", raw_session),
        };

        Ok(crunchy)
    })
});

pub async fn set_session(crunchy: Crunchyroll) -> anyhow::Result<()> {
    match crunchy.session_token().await {
        SessionToken::RefreshToken(refresh_token) => Ok(set_store(
            "session".into(),
            format!("refresh_token:{}", refresh_token),
        )?),
        SessionToken::EtpRt(etp_rt) => {
            Ok(set_store("session".into(), format!("etp_rt:{}", etp_rt))?)
        }
    }
}

pub fn has_session() -> bool {
    has_store("session".into())
}
