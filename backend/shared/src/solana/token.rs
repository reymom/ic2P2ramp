use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct TokenInfo {
    pub decimals: u8,
    /// UI symbol, e.g. "USDC" (optional but useful)
    pub symbol: String,
    /// Symbol used by your pricing layer (XRC/cache), e.g. "USDC"
    pub rate_symbol: String,
}
