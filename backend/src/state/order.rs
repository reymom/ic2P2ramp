use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

use super::common::PaymentProvider;

const MAX_ORDER_SIZE: u32 = 500;

#[derive(CandidType, Deserialize, Clone)]
pub struct Order {
    pub id: String,
    pub originator: Principal,
    pub fiat_amount: u64,
    pub currency_symbol: String,
    pub crypto_amount: u64,
    pub offramper_providers: Vec<PaymentProvider>,
    pub onramper_provider: Option<PaymentProvider>,
    pub offramper_address: String,
    pub onramper_address: Option<String>,
    pub locked: bool,
    pub proof_submitted: bool,
    pub chain_id: u64,
    pub token_type: String,
    pub payment_done: bool,
    pub removed: bool,
}

impl Storable for Order {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_ORDER_SIZE,
        is_fixed_size: false,
    };
}
