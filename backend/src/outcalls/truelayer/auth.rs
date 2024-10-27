use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::api::time;
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
    types::payment::truelayer,
};

#[derive(Serialize, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}

pub async fn get_access_token() -> Result<String> {
    if let Some((token, expiration)) = truelayer::get_token() {
        let current_time = time() / 1_000_000_000;
        ic_cdk::println!(
            "[truelayer.get_access_token] Cached token expiration: {}, current_time = {}",
            expiration,
            current_time
        );
        if current_time < expiration {
            return Ok(token);
        }
    }

    ic_cdk::println!("[truelayer.get_access_token] Fetching new token");
    let (client_id, client_secret, host_url) = read_state(|s| {
        (
            s.truelayer.client_id.clone(),
            s.truelayer.client_secret.clone(),
            s.truelayer.host_url.clone(),
        )
    });

    let request_headers = vec![HttpHeader {
        name: "Content-Type".to_string(),
        value: "application/x-www-form-urlencoded".to_string(),
    }];

    let body = format!(
        "grant_type=client_credentials&client_id={}&client_secret={}&scope=payments",
        client_id, client_secret
    );

    let request = CanisterHttpRequestArgument {
        url: format!("https://auth.{}/connect/token", host_url),
        method: HttpMethod::POST,
        body: Some(body.as_bytes().to_vec()),
        max_response_bytes: Some(4096),
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 21_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;

            ic_cdk::println!(
                "[truelayer.get_access_token] Raw response body: {}",
                str_body
            );

            let access_token_response: AccessTokenResponse = serde_json::from_str(&str_body)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;

            // Store the token and its expiration time in the state
            let expiration_time = access_token_response.expires_in + time() / 1_000_000_000;
            ic_cdk::println!(
                "[truelayer.get_access_token] New token expiration time: {}",
                expiration_time
            );
            truelayer::set_token(access_token_response.access_token.clone(), expiration_time);

            Ok(access_token_response.access_token)
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
