pub(crate) const OFFRAMPER_FIAT_FEE_DENOM: u64 = 40; // 2.5%
pub(crate) const ADMIN_CRYPTO_FEE_DENOM: u128 = 200; // 0.5%

pub fn get_fiat_fee(fiat_amount: u64) -> u64 {
    fiat_amount / OFFRAMPER_FIAT_FEE_DENOM
}

pub fn get_crypto_fee(crypto_amount: u128, blockchain_fees: u128) -> u128 {
    let admin_fee = crypto_amount / ADMIN_CRYPTO_FEE_DENOM;
    blockchain_fees + admin_fee
}
