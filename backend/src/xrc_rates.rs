use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::call_with_payment128;
use serde::Serialize;

const XRC_CANISTER_ID: &str = "uf6dk-hyaaa-aaaaq-qaaaq-cai";
const CYCLES_AMOUNT: u64 = 10_000_000_000;

#[derive(CandidType, Deserialize, Clone, Debug)]
enum AssetClass {
    Cryptocurrency,
    FiatCurrency,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
struct Asset {
    symbol: String,
    class: AssetClass,
}

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
enum ExchangeRateError {
    AnonymousPrincipalNotAllowed,
    Pending,
    CryptoBaseAssetNotFound,
    CryptoQuoteAssetNotFound,
    StablecoinRateNotFound,
    StablecoinRateTooFewRates,
    StablecoinRateZeroRate,
    ForexInvalidTimestamp,
    ForexBaseAssetNotFound,
    ForexQuoteAssetNotFound,
    ForexAssetsNotFound,
    RateLimited,
    NotEnoughCycles,
    FailedToAcceptCycles,
    InconsistentRatesReceived,
    Other { code: u32, description: String },
}

#[derive(CandidType, Deserialize, Clone, Debug)]
enum GetExchangeRateResult {
    Ok(ExchangeRate),
    Err(ExchangeRateError),
}

pub async fn get_exchange_rate(symbol: &str) -> Result<f64, String> {
    let request = GetExchangeRateRequest {
        base_asset: Asset {
            class: AssetClass::Cryptocurrency,
            symbol: symbol.to_string(),
        },
        quote_asset: Asset {
            class: AssetClass::FiatCurrency,
            symbol: "USD".to_string(),
        },
        timestamp: None,
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
    .map_err(|e| format!("Failed to call exchange rate canister: {:?}", e))?;

    match result.0 {
        GetExchangeRateResult::Ok(rate_response) => {
            let float_rate = rate_response.rate as f64;
            let float_divisor = 10u64.pow(rate_response.metadata.decimals) as f64;
            Ok(float_rate / float_divisor)
        }
        GetExchangeRateResult::Err(err) => {
            ic_cdk::println!("Exchange rate error: {:?}", err);
            Err(format!("Exchange rate error: {:?}", err))
        }
    }
}
