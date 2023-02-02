use crate::{Crunchyroll, Result};
use serde_json::json;

impl Crunchyroll {
    /// Verify a device with an code. Generally 6 characters long and used when logging in to non
    /// computer / smartphone devices like PlayStation, Xbox or Android TV.
    pub async fn verify_device(&self, code: String) -> Result<()> {
        let endpoint = "https://www.crunchyroll.com/auth/v1/device";
        self.executor
            .post(endpoint)
            .json(&json!({ "user_code": code }))
            .request_raw()
            .await?;
        Ok(())
    }
}
