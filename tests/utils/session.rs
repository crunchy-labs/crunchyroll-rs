#![allow(dead_code)]

use crate::utils::store::{Store, get_store, has_store, set_store};
use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::crunchyroll::{DeviceIdentifier, SessionToken};
use std::ops::Not;

pub static SESSION: Store<Crunchyroll> = Store::new(|| {
    Box::pin(async {
        let raw_session = get_store("session".into()).unwrap();
        let (raw_token, raw_device_identifier) = raw_session.split_once('\n').unwrap_or(("", ""));

        let (token_type, token) = raw_token.split_once(':').unwrap_or(("", ""));
        let device_identifier = raw_device_identifier.is_empty().not().then(|| {
            let mut split = raw_device_identifier.splitn(3, ':');
            DeviceIdentifier {
                device_id: split.next().unwrap_or_default().to_string(),
                device_type: split.next().unwrap_or_default().to_string(),
                device_name: split.next().map_or(Some("".to_string()), |dn| {
                    dn.is_empty().not().then_some(dn.to_string())
                }),
            }
        });

        let mut crunchyroll_builder = Crunchyroll::builder();
        if let Some(device_identifier) = device_identifier {
            crunchyroll_builder = crunchyroll_builder.device_identifier(device_identifier);
        }

        let crunchy = match token_type {
            "refresh_token" => crunchyroll_builder
                .login_with_refresh_token(token)
                .await
                .unwrap(),
            "etp_rt" => crunchyroll_builder.login_with_etp_rt(token).await.unwrap(),
            _ => panic!("invalid session '{raw_session}'"),
        };

        Ok(crunchy)
    })
});

pub async fn set_session(crunchy: Crunchyroll) -> anyhow::Result<()> {
    let token = match crunchy.session_token().await {
        SessionToken::RefreshToken(refresh_token) => format!("refresh_token:{refresh_token}"),
        SessionToken::EtpRt(etp_rt) => format!("etp_rt:{etp_rt}"),
        SessionToken::Anonymous => return Ok(()),
    };
    let device_identifier = crunchy.device_identifier().map_or("".to_string(), |di| {
        format!(
            "{}:{}:{}",
            di.device_id,
            di.device_type,
            di.device_name.unwrap_or("".to_string())
        )
    });

    set_store(
        "session".to_string(),
        format!("{token}\n{device_identifier}"),
    )?;
    Ok(())
}

pub fn has_session() -> bool {
    has_store("session".into())
}
