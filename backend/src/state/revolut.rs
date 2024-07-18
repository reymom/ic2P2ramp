use super::{mutate_state, read_state};

#[derive(Clone, Debug)]
pub struct RevolutState {
    pub access_token: Option<String>,
    pub token_expiration: Option<u64>,
    pub client_id: String,
    pub api_url: String,
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
