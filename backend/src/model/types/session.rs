use candid::{CandidType, Deserialize};

use crate::{
    management::random,
    model::errors::{Result, UserError},
};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Session {
    pub token: String,
    pub expires_at: u64, // nanoseconds
}

impl Session {
    pub(crate) const EXPIRATION_SECS: u64 = 43200; // 12h

    pub async fn new() -> Result<Self> {
        Ok(Session {
            token: random::generate_token().await?,
            expires_at: ic_cdk::api::time() + Self::EXPIRATION_SECS * 1_000_000_000,
        })
    }

    pub fn validate(&self, provided_token: &str) -> Result<()> {
        if self.token != provided_token {
            return Err(UserError::TokenInvalid.into());
        }
        if ic_cdk::api::time() >= self.expires_at {
            return Err(UserError::TokenExpired.into());
        }
        Ok(())
    }
}
