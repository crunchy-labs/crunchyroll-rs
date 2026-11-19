//! Internals for integration tests.

use crate::date;
use crate::{Crunchyroll, Result};

impl Crunchyroll {
    #[doc(hidden)]
    pub async fn refresh_jwt(&self) -> Result<()> {
        self.executor.session.write().await.session_expire = date::now();
        let _ = self
            .executor
            .auth_req(self.client().get("http://example.com"))
            .await?;
        Ok(())
    }
}
