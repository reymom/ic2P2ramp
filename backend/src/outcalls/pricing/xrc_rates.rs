use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::call::call_with_payment128;

use crate::{
    errors::{Result, SystemError},
    model::{errors::ExchangeRateError, types::exchange_rate::Asset},
};

const XRC_CANISTER_ID: &str = "uf6dk-hyaaa-aaaaq-qaaaq-cai";

#[derive(CandidType, Deserialize, Clone, Debug)]
struct GetExchangeRateRequest {
    base_asset: Asset,
    quote_asset: Asset,
    timestamp: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct ExchangeRateMetadata {
    decimals: u32,
    base_asset_num_received_rates: u64,
    base_asset_num_queried_sources: u64,
    quote_asset_num_received_rates: u64,
    quote_asset_num_queried_sources: u64,
    standard_deviation: u64,
    forex_timestamp: Option<u64>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct ExchangeRate {
    base_asset: Asset,
    quote_asset: Asset,
    timestamp: u64,
    rate: u64,
    metadata: ExchangeRateMetadata,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
enum GetExchangeRateResult {
    Ok(ExchangeRate),
    Err(ExchangeRateError),
}

pub(super) async fn get_xrc_exchange_rate(base_asset: Asset, quote_asset: Asset) -> Result<f64> {
    let request = GetExchangeRateRequest {
        base_asset,
        quote_asset,
        timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
    };

    // Every XRC call needs 1B cycles.
    let cycles: u128 = 1_000_000_000;
    let result: (GetExchangeRateResult,) = call_with_payment128(
        Principal::from_text(XRC_CANISTER_ID).expect(" xrc canister id should be defined "),
        "get_exchange_rate",
        (request,),
        cycles,
    )
    .await
    .map_err(|e| SystemError::CanisterCallError(format!("{:?}", e)))?;

    match result.0 {
        GetExchangeRateResult::Ok(rate_response) => {
            let float_rate = rate_response.rate as f64;
            let float_divisor = 10u64.pow(rate_response.metadata.decimals) as f64;
            Ok(float_rate / float_divisor)
        }
        GetExchangeRateResult::Err(err) => Err(SystemError::ExchangeRateError(err).into()),
    }
}
