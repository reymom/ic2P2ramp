use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
    types::ordiscan::OrdiscanRunePrice,
};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use std::string::String;

pub(super) async fn fetch_rune_price(rune_name: &str) -> Result<OrdiscanRunePrice> {
    let (api_url, api_key, proxy_url) = read_state(|s| {
        (
            s.ordiscan.api_url.clone(),
            s.ordiscan.api_key.clone(),
            s.proxy_url.clone(),
        )
    });

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", api_key),
        },
        HttpHeader {
            name: "x-forwarded-host".to_string(),
            value: api_url,
        },
        HttpHeader {
            name: "idempotency-key".to_string(),
            value: format!("ordiscan-price-key-{}-{}", rune_name, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/v1/rune/{}/market", proxy_url, rune_name),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(8_000),
        transform: None,
        headers: request_headers,
    };

    let cycles = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let response_str =
                String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
            ic_cdk::println!("[fetch_rune_price] response: {}", response_str);
            ic_cdk::println!("[fetch_rune_price] status: {}", response.status);
            let rune_price: OrdiscanRunePrice = serde_json::from_str(&response_str)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;

            Ok(rune_price)
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
