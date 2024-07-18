use serde::{Deserialize, Serialize};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::api::time;

use crate::{
    errors::{RampError, Result},
    state::{read_state, revolut},
};

#[derive(Serialize, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

pub async fn get_revolut_access_token() -> Result<String> {
    if let Some((token, expiration)) = revolut::get_revolut_token() {
        let current_time = time() / 1_000_000_000;
        ic_cdk::println!(
            "[get_revolut_access_token] Cached token expiration: {}",
            expiration
        );
        ic_cdk::println!("[get_revolut_access_token] current_time = {}", current_time);
        if current_time < expiration {
            return Ok(token);
        }
    }

    ic_cdk::println!("[get_revolut_access_token] Fetching new token from Revolut");
    let (client_id, api_url) =
        read_state(|s| (s.revolut.client_id.clone(), s.revolut.api_url.clone()));
    ic_cdk::println!(
        "[get_revolut_access_token] client_id = {}, api_url = {}",
        client_id,
        api_url
    );

    let request_headers = vec![HttpHeader {
        name: "Content-Type".to_string(),
        value: "application/x-www-form-urlencoded".to_string(),
    }];

    let request_body = format!(
        "grant_type=client_credentials&scope=accounts&client_id={}",
        client_id
    );

    let request = CanisterHttpRequestArgument {
        url: format!("{}/token", api_url),
        method: HttpMethod::POST,
        body: Some(request_body.as_bytes().to_vec()),
        max_response_bytes: None,
        transform: None,
        headers: request_headers,
    };

    ic_cdk::println!("[get_revolut_access_token] request = {:?}", request);

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            ic_cdk::println!("[get_revolut_access_token] Response body: {}", str_body);

            let access_token_response: AccessTokenResponse = serde_json::from_str(&str_body)
                .map_err(|e| RampError::ParseError(e.to_string()))?;

            // Store the token and its expiration time in the state
            let current_time = time() / 1_000_000_000;
            let expiration_time = current_time + access_token_response.expires_in;
            ic_cdk::println!(
                "[get_revolut_access_token] New token expiration time: {}",
                expiration_time
            );
            revolut::set_revolut_token(access_token_response.access_token.clone(), expiration_time);

            Ok(access_token_response.access_token)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
