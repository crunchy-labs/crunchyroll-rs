//! Internal functions for integration tests.

use crate::{Crunchyroll, Result};
use chrono::Utc;

impl Crunchyroll {
    pub async fn refresh_jwt(&self) -> Result<()> {
        self.executor.session.write().await.session_expire = Utc::now();
        let _ = self
            .executor
            .auth_req(self.client().get("http://example.com"))
            .await?;
        Ok(())
    }
}
