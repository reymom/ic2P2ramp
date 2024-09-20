use candid::{CandidType, Deserialize};

use crate::model::memory::heap::{mutate_state, read_state};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct PayPalState {
    pub access_token: Option<String>,
    pub token_expiration: Option<u64>,
    pub client_id: String,
    pub client_secret: String,
    pub api_url: String,
}

pub fn get_paypal_token() -> Option<(String, u64)> {
    read_state(|s| {
        if let (Some(token), Some(expiration)) =
            (s.paypal.access_token.clone(), s.paypal.token_expiration)
        {
            Some((token, expiration))
        } else {
            None
        }
    })
}

pub fn set_paypal_token(token: String, expiration: u64) {
    mutate_state(|s| {
        s.paypal.access_token = Some(token);
        s.paypal.token_expiration = Some(expiration);
    });
}
