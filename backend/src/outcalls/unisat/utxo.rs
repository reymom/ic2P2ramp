use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use std::str::FromStr;

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
    types::unisat::UnisatRuneBalanceResponse,
};

pub async fn fetch_rune_utxo_balance(txid: &str, vout: u32) -> Result<u32> {
    let (api_url, api_key, proxy_url) = read_state(|s| {
        (
            s.unisat.api_url.clone(),
            s.unisat.api_key.clone(),
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
            value: format!("unisat-utxo-key-{}-{}", txid, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!(
            "{}/v1/indexer/runes/utxo/{}/{}/balance",
            proxy_url, txid, vout
        ),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(10_000),
        transform: None,
        headers: request_headers,
    };

    let cycles = 10_000_000_000;
    let (response,) = http_request(request, cycles)
        .await
        .map_err(|(code, msg)| SystemError::HttpRequestError(code as u64, msg))?;

    let response_str = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
    let tx_response: UnisatRuneBalanceResponse =
        serde_json::from_str(&response_str).map_err(|e| SystemError::ParseError(e.to_string()))?;

    if tx_response.data.is_empty() {
        return Ok(0);
    }

    Ok(u32::from_str(&tx_response.data[0].amount)
        .map_err(|e| SystemError::ParseIntError(e.to_string()))?)
}
