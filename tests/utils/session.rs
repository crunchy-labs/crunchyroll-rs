use std::error::Error;
use crunchyroll_rs::Crunchyroll;
use crate::utils::store::{get_store, has_store, set_store, Store};

pub static SESSION: Store<Crunchyroll> = Store::new(|| Box::pin(async {
    let crunchy = Crunchyroll::new()
        .login_with_etp_rt(get_store("session".into()).unwrap())
        .await?;
    Ok(crunchy)
}));

pub fn set_session(crunchy: Crunchyroll) -> anyhow::Result<()> {
    Ok(set_store("session".into(), crunchy.config().refresh_token)?)
}

pub fn has_session() -> bool {
    has_store("session".into())
}
