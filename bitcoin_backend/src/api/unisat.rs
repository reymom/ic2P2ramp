use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use crate::memory::heap::state::read_state;
use crate::model::types::{
    errors::Result, runes::RuneID, unisat::UnisatRuneUTXOsResponse, utxo::RuneUTXOEntry,
};
use crate::types::errors::SystemError;

pub async fn fetch_rune_utxos(address: &str, rune_id: RuneID) -> Result<Vec<RuneUTXOEntry>> {
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
            value: format!("unisat-runes-key-{}-{}", address, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!(
            "{}/v1/indexer/address/{}/runes/{}/utxo",
            proxy_url, address, rune_id
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
    let utxos_response: UnisatRuneUTXOsResponse =
        serde_json::from_str(&response_str).map_err(|e| SystemError::ParseError(e.to_string()))?;

    if utxos_response.data.utxo.is_empty() {
        return Ok(Vec::new());
    }

    Ok(utxos_response.into_rune_utxo_entries(&rune_id.to_string()))
}
