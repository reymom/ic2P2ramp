use crate::{
    errors::{RampError, Result},
    state::read_state,
};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use super::auth::get_revolut_access_token;

pub async fn get_revolut_transactions(account_id: &str) -> Result<String> {
    let access_token = get_revolut_access_token().await?;

    let api_url = read_state(|s| s.revolut.api_url.clone());

    let request_headers = vec![
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
        HttpHeader {
            name: "x-fapi-financial-id".to_string(),
            value: "001580000103UAvAAM".to_string(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/accounts/{}/transactions", api_url, account_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            ic_cdk::println!("[get_revolut_transactions] Response body: {}", str_body);
            Ok(str_body)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
