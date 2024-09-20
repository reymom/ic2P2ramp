use core::fmt;

use candid::{CandidType, Deserialize};

use crate::model::memory::heap::{mutate_state, read_state};

#[derive(Clone, CandidType, Deserialize)]
pub struct RevolutState {
    pub access_token: Option<String>,
    pub token_expiration: Option<u64>,
    pub client_id: String,
    pub api_url: String,
    pub proxy_url: String,
    pub private_key_der: Vec<u8>,
    pub kid: String,
    pub tan: String,
}

impl fmt::Debug for RevolutState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevolutConfig")
            .field("client_id", &self.client_id)
            .field("api_url", &self.api_url)
            .field("proxy_url", &self.proxy_url)
            .field("kid", &self.kid)
            .field("tan", &self.tan)
            .finish()
    }
}

pub fn get_revolut_token() -> Option<(String, u64)> {
    read_state(|s| {
        if let (Some(token), Some(expiration)) =
            (s.revolut.access_token.clone(), s.revolut.token_expiration)
        {
            Some((token, expiration))
        } else {
            None
        }
    })
}

pub fn set_revolut_token(token: String, expiration: u64) {
    mutate_state(|s| {
        s.revolut.access_token = Some(token);
        s.revolut.token_expiration = Some(expiration);
    });
}
