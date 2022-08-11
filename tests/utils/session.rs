use std::error::Error;
use once_cell::sync::OnceCell;
use crunchyroll_rs::Crunchyroll;
use crate::utils::store::{get_store, has_store, set_store};

const SESSION: OnceCell<Crunchyroll> = OnceCell::new();

pub async fn get_session() -> Result<Crunchyroll, Box<dyn Error>> {
    if let Some(session) = SESSION.get() {
        Ok(session.clone())
    } else {
        let session = Crunchyroll::new().login_with_etp_rt(get_store("session".into())?).await?;
        SESSION.set(session.clone()).unwrap();
        Ok(session)
    }
}

pub fn set_session(crunchy: Crunchyroll) -> Result<(), Box<dyn Error>> {
    Ok(set_store("session".into(), crunchy.config().refresh_token)?)
}

pub fn has_session() -> bool {
    has_store("session".into())
}
