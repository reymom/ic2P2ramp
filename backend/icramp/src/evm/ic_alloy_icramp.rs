use alloy::primitives::Address;
use alloy::sol;
use alloy::sol_types::SolCall;
use evm_rpc_canister_types::{EthSepoliaService, HttpHeader as HttpHeaderEvm, RpcApi, RpcServices};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, http_request,
};
use serde_json::json;

use crate::model::errors::SystemError;
use crate::{
    errors::{BlockchainError, Result},
    model::types::evm::chains::get_rpc_providers,
};

sol! {
    #[sol(rpc)]
    contract IcRamp {
        function getDeposit(address _offramper, address _token) external view returns (uint256);
    }
}

pub fn encode_get_deposit(offramper: &str, token: &str) -> Result<Vec<u8>> {
    let off = offramper
        .parse::<Address>()
        .map_err(|e| BlockchainError::EthersAbiError(format!("offramper parse: {:?}", e)))?;
    let tok = token
        .parse::<Address>()
        .map_err(|e| BlockchainError::EthersAbiError(format!("token parse: {:?}", e)))?;

    // This uses ic-alloy’s generated ABI to build calldata
    Ok(IcRamp::getDepositCall {
        _offramper: off,
        _token: tok,
    }
    .abi_encode())
}

pub async fn get_deposit_via_ic_alloy(
    chain_id: u64,
    icramp_address: &str,
    offramper: &str,
    token: &str,
) -> Result<u128> {
    let proxy_url = crate::model::memory::heap::read_state(|s| s.proxy_url.clone());
    let (rpc_url, ..) = resolve_rpc_url_for_chain(chain_id)?;
    let (base_url, endpoint) = split_url_for_proxy(&rpc_url);

    let data = encode_get_deposit(offramper, token)?;
    let data_hex = format!("0x{}", hex::encode(data));

    let params = json!([{
        "to": icramp_address,
        "data": data_hex,
    }, "latest"]);

    let request = CanisterHttpRequestArgument {
        url: format!("{}/{}", proxy_url, endpoint),
        method: HttpMethod::POST,
        body: Some(
            json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": params,
                "id": 1
            })
            .to_string()
            .into_bytes(),
        ),
        max_response_bytes: Some(2048),
        transform: None,
        headers: vec![
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
            HttpHeader {
                name: "x-forwarded-host".to_string(),
                value: base_url.to_string(),
            },
        ],
    };

    let cycles = 10_000_000_000;
    let (response,) = http_request(request, cycles)
        .await
        .map_err(|(code, msg)| SystemError::HttpRequestError(code as u64, msg.to_string()))?;
    if response.status.ne(&candid::Nat::from(200u32)) {
        return Err(BlockchainError::EvmExecutionReverted(
            0,
            "HTTP error in getDeposit".to_string(),
        )
        .into());
    }

    let str_body = str::from_utf8(&response.body)
        .map_err(|_| BlockchainError::EvmExecutionReverted(0, "utf8".to_string()))?;
    let json_response: serde_json::Value = serde_json::from_str(str_body)
        .map_err(|e| BlockchainError::EvmExecutionReverted(0, e.to_string()))?;

    if let Some(result) = json_response.get("result").and_then(|r| r.as_str()) {
        let clean = result.trim_start_matches("0x");
        let value = u128::from_str_radix(clean, 16)
            .map_err(|_| BlockchainError::EvmExecutionReverted(0, "parse u128".to_string()))?;
        Ok(value)
    } else {
        Err(BlockchainError::EvmExecutionReverted(0, "missing result".to_string()).into())
    }
}

fn resolve_rpc_url_for_chain(chain_id: u64) -> Result<(String, Vec<HttpHeaderEvm>)> {
    let services = get_rpc_providers(chain_id)?;

    match services {
        RpcServices::Custom { services, .. } => {
            let api: &RpcApi = services
                .first()
                .ok_or(BlockchainError::RpcProviderNotFound)?;
            Ok((api.url.clone(), api.headers.clone().unwrap_or_default()))
        }

        RpcServices::EthSepolia(Some(list)) => {
            // Pick the first supported service in the order configured
            for svc in list {
                let url = match svc {
                    EthSepoliaService::Sepolia => Some("https://rpc.sepolia.org".to_string()),
                    EthSepoliaService::PublicNode => {
                        Some("https://ethereum-sepolia-rpc.publicnode.com".to_string())
                    }
                    EthSepoliaService::BlockPi => {
                        Some("https://ethereum-sepolia.blockpi.network/v1/rpc/public".to_string())
                    }
                    EthSepoliaService::Ankr => Some("https://rpc.ankr.com/eth_sepolia".to_string()),
                    EthSepoliaService::Alchemy => None,
                };

                if let Some(url) = url {
                    return Ok((url, Vec::new()));
                }
            }

            Err(BlockchainError::RpcProviderNotFound.into())
        }

        // You can extend these as you hook more chains,
        // for now we fail fast to avoid silent misconfig.
        _ => Err(BlockchainError::RpcProviderNotFound.into()),
    }
}

fn split_url_for_proxy(full_url: &str) -> (String, String) {
    let without_proto = full_url
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    if let Some((host, path)) = without_proto.split_once('/') {
        (host.to_string(), format!("/{}", path))
    } else {
        (without_proto.to_string(), "/".to_string())
    }
}
