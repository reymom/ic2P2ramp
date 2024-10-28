use serde::{Deserialize, Serialize};
use serde_json::to_vec;

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
};

#[derive(Serialize, Deserialize, Debug, candid::CandidType)]
pub struct ProvidersResponse {}

pub fn parse_providers_response(response_body: Vec<u8>) -> Result<ProvidersResponse> {
    let str_body = String::from_utf8(response_body).map_err(|_| SystemError::Utf8Error)?;
    ic_cdk::println!("[parse_providers_response] str_body = {}", str_body);

    if let Ok(error_response) = serde_json::from_str::<super::ErrorResponse>(&str_body) {
        return Err(SystemError::ParseError(error_response.error))?;
    }

    let response: ProvidersResponse =
        serde_json::from_str(&str_body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    Ok(response)
}

#[derive(Serialize)]
pub struct GetProvidersRequest {}

pub async fn get_providers(country: &str) -> Result<ProvidersResponse> {
    let access_token = super::auth::get_access_token().await?;
    let host_url = read_state(|s| s.truelayer.host_url.clone());

    let payload = GetProvidersRequest {};
    let body = to_vec(&payload).map_err(|e| SystemError::ParseError(e.to_string()))?;

    let headers = vec![
        HttpHeader {
            name: "content-type".to_string(),
            value: "application/json; charset=UTF-8".to_string(),
        },
        HttpHeader {
            name: "accept".to_string(),
            value: "application/json; charset=UTF-8".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("https://api.{}/v3/payments-providers/search", host_url),
        method: HttpMethod::POST,
        body: Some(body),
        max_response_bytes: Some(8192),
        transform: None,
        headers,
    };

    let cycles: u128 = 21_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => parse_providers_response(response.body),
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
