/// Begins with an underscore because this must be the first file to be called
mod utils;

use crunchyroll_rs::Crunchyroll;
use std::env;

#[tokio::test]
async fn login_with_credentials() {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchy = Crunchyroll::builder()
        .login_with_credentials(user, password)
        .await;

    assert_result!(crunchy);

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_refresh_token() {
    let refresh_token =
        env::var("REFRESH_TOKEN").expect("'REFRESH_TOKEN' environment variable not found");

    let crunchy = Crunchyroll::builder()
        .login_with_refresh_token(refresh_token)
        .await;

    assert_result!(crunchy);

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_etp_rt() {
    let etp_rt = env::var("ETP_RT").expect("'ETP_RT' environment variable not found");

    let crunchy = Crunchyroll::builder().login_with_etp_rt(etp_rt).await;

    assert_result!(crunchy);

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}

#[tokio::test]
async fn login_with_session_id() {
    let session_id = env::var("SESSION_ID").expect("'SESSION_ID' environment variable not found");

    let crunchy = Crunchyroll::builder()
        .login_with_session_id(session_id)
        .await;

    assert_result!(crunchy);

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).await.unwrap()
    }
}
