use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

use super::common::PaymentProvider;

const MAX_USER_SIZE: u32 = 100;

#[derive(CandidType, Deserialize, Clone)]
pub struct User {
    pub evm_address: String,
    pub payment_providers: Vec<PaymentProvider>,
    pub offramped_amount: u64,
    pub score: u32,
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_USER_SIZE,
        is_fixed_size: false,
    };
}
