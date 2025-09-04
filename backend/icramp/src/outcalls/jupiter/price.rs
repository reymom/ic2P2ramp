use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, http_request,
};

pub async fn get_usd_price_by_mint(mint: &str) -> Result<f64> {
    let proxy_url = read_state(|s| s.proxy_url.clone());
    let api_base = "https://lite-api.jup.ag".to_string();

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "x-forwarded-host".to_string(),
            value: api_base,
        },
        HttpHeader {
            name: "idempotency-key".to_string(),
            value: format!("jupiter-price-key-{}-{}", mint, ic_cdk::api::time()).to_string(),
        },
    ];

    let req = CanisterHttpRequestArgument {
        url: format!("{}/price/v3?ids={}", proxy_url, mint),
        method: HttpMethod::GET,
        headers: request_headers,
        body: None,
        max_response_bytes: Some(8_000),
        transform: None,
    };

    let (resp,) = http_request(req, 10_000_000_000)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let body = String::from_utf8(resp.body).map_err(|_| SystemError::Utf8Error)?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| SystemError::ParseError(e.to_string()))?;

    // Response shape: { "<mint>": { "usdPrice": <f64>, ... } }
    let usd = v
        .get(mint)
        .and_then(|o| o.get("usdPrice"))
        .and_then(|n| n.as_f64())
        .ok_or_else(|| SystemError::ParseError("missing usdPrice".into()))?;

    Ok(usd)
}
