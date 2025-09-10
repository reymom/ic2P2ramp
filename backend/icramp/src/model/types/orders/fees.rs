use candid::CandidType;
use serde::{Deserialize, Serialize};

pub(crate) const OFFRAMPER_FIAT_FEE_DENOM: u64 = 40; // 2.5%
pub(crate) const ADMIN_CRYPTO_FEE_DENOM: u128 = 200; // 0.5%

#[derive(CandidType, Deserialize, Serialize)]
pub struct FeeQuote {
    pub blockchain_fee: u128, // network fee in token base units
    pub admin_fee: u128,      // crypto_amount / ADMIN_CRYPTO_FEE_DENOM
    pub total_fee: u128,      // blockchain_fee + admin_fee
}

pub fn get_admin_fee(crypto_amount: u128) -> u128 {
    crypto_amount / ADMIN_CRYPTO_FEE_DENOM
}

pub fn get_fiat_fee(fiat_amount: u64) -> u64 {
    fiat_amount / OFFRAMPER_FIAT_FEE_DENOM
}

pub fn get_crypto_fee(crypto_amount: u128, blockchain_fees: u128) -> u128 {
    let admin_fee = crypto_amount / ADMIN_CRYPTO_FEE_DENOM;
    blockchain_fees + admin_fee
}
