use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct OrdiscanState {
    pub api_url: String,
    pub api_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrdiscanRunePrice {
    pub data: RuneMarketData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuneMarketData {
    pub price_in_sats: f64,
    pub price_in_usd: f64,
    pub market_cap_in_btc: f64,
    pub market_cap_in_usd: f64,
}
