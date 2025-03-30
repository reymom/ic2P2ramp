use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
    types::unisat::{UnisatTxOut, UnisatTxOutResponse, UnisatTxStatusResponse},
};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

pub async fn fetch_unisat_tx_status(txid: &str) -> Result<bool> {
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
            value: format!("ordiscan-tx-key-{}-{}", txid, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/v1/indexer/tx/{}", proxy_url, txid),
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
    let tx_response: UnisatTxStatusResponse =
        serde_json::from_str(&response_str).map_err(|e| SystemError::ParseError(e.to_string()))?;

    Ok(tx_response.data.confirmations > 2)
}

pub async fn fetch_unisat_tx_outs(txid: &str) -> Result<Vec<UnisatTxOut>> {
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
            value: format!("unisat-tx-key-{}-{}", txid, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/v1/indexer/tx/{}/outs", proxy_url, txid),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(16_000),
        transform: None,
        headers: request_headers,
    };

    let cycles = 10_000_000_000;
    let (response,) = http_request(request, cycles)
        .await
        .map_err(|(code, msg)| SystemError::HttpRequestError(code as u64, msg))?;

    let response_str = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
    let tx_response: UnisatTxOutResponse =
        serde_json::from_str(&response_str).map_err(|e| SystemError::ParseError(e.to_string()))?;

    Ok(tx_response.data)
}
