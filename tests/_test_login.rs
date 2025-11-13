#![allow(clippy::bool_assert_comparison)]
/// Begins with an underscore because this must be the first file to be called
mod utils;

use crate::utils::SESSION;
use crunchyroll_rs::Crunchyroll;
use crunchyroll_rs::crunchyroll::DeviceIdentifier;
use std::env;

#[tokio::test]
async fn login_with_credentials() {
    let email = env::var("EMAIL").expect("'EMAIL' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");
    let is_premium = env::var("IS_PREMIUM")
        .ok()
        .map(|e| e.parse::<bool>().unwrap());

    let crunchy = Crunchyroll::builder()
        .login_with_credentials(email, password, DeviceIdentifier::default())
        .await;

    assert_result!(crunchy);
    if let Some(is_premium) = is_premium {
        assert_eq!(crunchy.as_ref().unwrap().premium().await, is_premium)
    }

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_refresh_token() {
    let refresh_token =
        env::var("REFRESH_TOKEN").expect("'REFRESH_TOKEN' environment variable not found");
    let is_premium = env::var("IS_PREMIUM")
        .ok()
        .map(|e| e.parse::<bool>().unwrap());

    let crunchy = Crunchyroll::builder()
        .login_with_refresh_token(refresh_token, DeviceIdentifier::default())
        .await;

    assert_result!(crunchy);
    if let Some(is_premium) = is_premium {
        assert_eq!(crunchy.as_ref().unwrap().premium().await, is_premium)
    }

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_refresh_token_profile_id() {
    let refresh_token =
        env::var("REFRESH_TOKEN").expect("'REFRESH_TOKEN' environment variable not found");
    let profile_id = env::var("PROFILE_ID").expect("'PROFILE_ID' environment variable not found");
    let is_premium = env::var("IS_PREMIUM")
        .ok()
        .map(|e| e.parse::<bool>().unwrap());

    let crunchy = Crunchyroll::builder()
        .login_with_refresh_token_profile_id(
            &refresh_token,
            &profile_id,
            DeviceIdentifier::default(),
        )
        .await;

    assert_result!(crunchy);
    assert_eq!(crunchy.as_ref().unwrap().profile_id().await, profile_id);
    if let Some(is_premium) = is_premium {
        assert_eq!(crunchy.as_ref().unwrap().premium().await, is_premium)
    }

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_etp_rt() {
    let etp_rt = env::var("ETP_RT").expect("'ETP_RT' environment variable not found");
    let etp_rt_device_id =
        env::var("ETP_RT_DEVICE_ID").expect("'ETP_RT_DEVICE_ID' environment variable not found");
    let etp_rt_device_type = env::var("ETP_RT_DEVICE_TYPE")
        .expect("'ETP_RT_DEVICE_TYPE' environment variable not found");
    let etp_rt_device_name = env::var("ETP_RT_DEVICE_TYPE").ok();
    let is_premium = env::var("IS_PREMIUM")
        .ok()
        .map(|e| e.parse::<bool>().unwrap());

    let crunchy = Crunchyroll::builder()
        .login_with_etp_rt(
            etp_rt,
            DeviceIdentifier {
                device_id: etp_rt_device_id,
                device_type: etp_rt_device_type,
                device_name: etp_rt_device_name,
            },
        )
        .await;

    assert_result!(crunchy);
    if let Some(is_premium) = is_premium {
        assert_eq!(crunchy.as_ref().unwrap().premium().await, is_premium)
    }

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_anonymously() {
    let crunchy = Crunchyroll::builder()
        .login_anonymously(DeviceIdentifier::default())
        .await;

    assert_result!(crunchy);
    assert_eq!(crunchy.as_ref().unwrap().premium().await, false);

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

/// Prefixed with `z` to run last.
#[cfg(feature = "__test")]
#[tokio::test]
async fn z_expired_token() {
    let crunchy = SESSION.get().await.unwrap();
    assert_result!(crunchy.refresh_jwt().await)
}
