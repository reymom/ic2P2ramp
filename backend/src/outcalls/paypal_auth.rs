use base64::{engine::general_purpose, Engine as _};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{RampError, Result},
    state::read_state,
};

#[derive(Serialize, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

pub async fn get_paypal_access_token() -> Result<String> {
    let (client_id, client_secret) = read_state(|s| (s.client_id.clone(), s.client_secret.clone()));
    let credentials = general_purpose::STANDARD.encode(format!("{}:{}", client_id, client_secret));

    let request_headers = vec![
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Basic {}", credentials),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/x-www-form-urlencoded".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: "https://api-m.sandbox.paypal.com/v1/oauth2/token".to_string(),
        method: HttpMethod::POST,
        body: Some("grant_type=client_credentials".as_bytes().to_vec()),
        max_response_bytes: None,
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            let access_token_response: AccessTokenResponse = serde_json::from_str(&str_body)
                .map_err(|e| RampError::ParseError(e.to_string()))?;

            Ok(access_token_response.access_token)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
