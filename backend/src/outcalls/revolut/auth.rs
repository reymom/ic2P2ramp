use serde::{Deserialize, Serialize};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::api::time;

use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
    types::payment::revolut,
};

#[derive(Serialize, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_at: u64,
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

    let api_url = read_state(|s| s.revolut.proxy_url.clone());

    let request_headers = vec![HttpHeader {
        name: "Content-Type".to_string(),
        value: "application/json".to_string(),
    }];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/revolut/token", api_url),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(1024), // content-length is 576 bytes
        transform: None,
        headers: request_headers,
    };

    ic_cdk::println!("[get_revolut_access_token] request = {:?}", request);

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
            ic_cdk::println!("[get_revolut_access_token] Response body: {}", str_body);

            let access_token_response: AccessTokenResponse = serde_json::from_str(&str_body)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;

            // Store the token and its expiration time in the state
            let expiration_time = access_token_response.expires_at;
            ic_cdk::println!(
                "[get_revolut_access_token] New token expiration time: {}",
                expiration_time
            );
            revolut::set_revolut_token(access_token_response.access_token.clone(), expiration_time);

            Ok(access_token_response.access_token)
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
