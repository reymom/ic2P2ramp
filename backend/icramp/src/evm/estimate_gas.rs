use core::str;
use std::str::FromStr;

use candid::Nat;
use evm_rpc_canister_types::RpcServices;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use num_traits::ToPrimitive;
use serde::Serialize;
use serde_json::json;

use crate::model::{
    errors::{BlockchainError, Result, SystemError},
    memory::heap::read_state,
};

#[derive(Serialize, Debug)]
pub struct EstimateGasParams {
    from: String,
    to: String,
    value: String,
    data: Option<String>,
}

impl EstimateGasParams {
    pub fn new(from: Option<String>, to: String, value: Option<String>, data: Vec<u8>) -> Self {
        let from_address =
            from.unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());
        let value_amount = value.unwrap_or_else(|| "0x0".to_string());
        let data_hex = format!("0x{}", hex::encode(data));

        EstimateGasParams {
            from: from_address,
            to,
            value: value_amount,
            data: Some(data_hex),
        }
    }
}

pub async fn estimate_gas(chain_id: u64, params: EstimateGasParams) -> Result<Option<u64>> {
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let rpc_provider_url = match get_rpc_provider(chain_id)? {
        Some(rpc_provider) => rpc_provider,
        None => return Ok(None),
    }
    .replace("https://", "");
    let (base_url, endpoint) = if let Some((domain, path)) = rpc_provider_url.split_once('/') {
        (domain, format!("{}", path))
    } else {
        (rpc_provider_url.as_str(), "/".to_string())
    };

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "x-forwarded-host".to_string(),
            value: base_url.to_string(),
        },
        HttpHeader {
            name: "idempotency-key".to_string(),
            value: format!("estimate-gas-{}-{}", chain_id, ic_cdk::api::time()).to_string(),
        },
    ];

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": [params],
        "id": 1
    })
    .to_string()
    .into_bytes();

    let request = CanisterHttpRequestArgument {
        url: format!("{}/{}", proxy_url, endpoint),
        method: HttpMethod::POST,
        body: Some(request_body),
        max_response_bytes: Some(2048),
        transform: None,
        headers: request_headers,
    };

    let cycles = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            if response
                .status
                .ne(&Nat::from_str("200").unwrap_or_default())
            {
                return Err(SystemError::HttpRequestError(
                    response.status.0.to_u64().unwrap_or_default(),
                    "HTTP error".to_string(),
                ))?;
            }

            let str_body = str::from_utf8(&response.body).map_err(|_| SystemError::Utf8Error)?;

            let json_response: serde_json::Value = serde_json::from_str(str_body)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;
            ic_cdk::println!("[estimate_gas] json_response = {}", json_response);

            if let Some(error) = json_response.get("error") {
                let error_message = error["message"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string();
                let error_code = error["code"].as_i64().unwrap_or(0);
                return Err(
                    BlockchainError::EvmExecutionReverted(error_code, error_message).into(),
                );
            }

            if let Some(gas_estimate) = json_response["result"].as_str() {
                let gas_estimate = u64::from_str_radix(gas_estimate.trim_start_matches("0x"), 16)
                    .map_err(|_| BlockchainError::GasEstimationFailed)?;
                Ok(Some(gas_estimate))
            } else {
                Err(BlockchainError::GasEstimationFailed)?
            }
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}

fn get_rpc_provider(chain_id: u64) -> Result<Option<String>> {
    read_state(|state| {
        let chain_state = state
            .chains
            .get(&chain_id)
            .ok_or(BlockchainError::ChainIdNotFound(chain_id))?;

        match &chain_state.rpc_services {
            RpcServices::Custom { services, .. } => {
                if let Some(primary_service) = services.first() {
                    Ok(Some(primary_service.url.clone()))
                } else {
                    Err(BlockchainError::RpcProviderNotFound)?
                }
            }
            _ => Ok(None),
        }
    })
}
