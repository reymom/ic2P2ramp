use core::fmt;

use candid::{CandidType, Deserialize};

use crate::model::memory::heap::{mutate_state, read_state};

#[derive(CandidType, Deserialize, Clone)]
pub struct TrueLayerState {
    pub access_token: Option<String>,
    pub token_expiration: Option<u64>,
    pub client_id: String,
    pub client_secret: String,
    pub kid: String,
    pub private_key: Vec<u8>,
    pub host_url: String,
    pub proxy_url: String,
}

impl fmt::Debug for TrueLayerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RevolutState")
            .field("access_token", &self.access_token)
            .field("token_expiration", &self.token_expiration)
            .field("client_id", &self.client_id)
            .field("client_secret", &self.client_secret)
            .field("kid", &self.kid)
            .field("host_url", &self.host_url)
            .field("proxy_url", &self.proxy_url)
            .finish()
    }
}

impl TrueLayerState {
    pub fn new(
        client_id: String,
        client_secret: String,
        kid: String,
        private_key: Vec<u8>,
        host_url: String,
        proxy_url: String,
    ) -> Self {
        TrueLayerState {
            access_token: None,
            token_expiration: None,
            client_id,
            client_secret,
            kid,
            private_key,
            host_url,
            proxy_url,
        }
    }
}

pub fn get_token() -> Option<(String, u64)> {
    read_state(|s| {
        if let (Some(token), Some(expiration)) = (
            s.truelayer.access_token.clone(),
            s.truelayer.token_expiration,
        ) {
            Some((token, expiration))
        } else {
            None
        }
    })
}

pub fn set_token(token: String, expiration: u64) {
    mutate_state(|s| {
        s.truelayer.access_token = Some(token);
        s.truelayer.token_expiration = Some(expiration);
    });
}
