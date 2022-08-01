use std::error::Error;
use crunchyroll_rs::Crunchyroll;
use crate::utils::store::{get_store, has_store, set_store};

pub async fn get_session() -> Result<Crunchyroll, Box<dyn Error>> {
    Ok(Crunchyroll::new().login_with_etp_rt(get_store("session".into())?).await?)
}

pub fn set_session(crunchy: Crunchyroll) -> Result<(), Box<dyn Error>> {
    Ok(set_store("session".into(), crunchy.config.refresh_token)?)
}

pub fn has_session() -> bool {
    has_store("session".into())
}
