use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, fmt};

use super::common::PaymentProvider;

const MAX_ORDER_SIZE: u32 = 500;

#[derive(CandidType, Deserialize, Clone)]
pub enum OrderState {
    Created(Order),
    Locked(LockedOrder),
    Completed(String),
    Cancelled(String),
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderState::Created(_) => write!(f, "Created"),
            OrderState::Locked(_) => write!(f, "Locked"),
            OrderState::Completed(_) => write!(f, "Completed"),
            OrderState::Cancelled(_) => write!(f, "Cancelled"),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Order {
    pub id: String,
    pub originator: Principal,
    pub created_at: u64,
    pub fiat_amount: u64,
    pub currency_symbol: String,
    pub crypto_amount: u64,
    pub offramper_providers: Vec<PaymentProvider>,
    pub offramper_address: String,
    pub chain_id: u64,
    pub token_address: Option<String>,
}

impl Order {
    pub fn lock(self, onramper_provider: PaymentProvider, onramper_address: String) -> LockedOrder {
        LockedOrder {
            base: self,
            onramper_address,
            onramper_provider,
            proof_submitted: false,
            payment_done: false,
            locked_at: time(),
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LockedOrder {
    pub base: Order,
    pub onramper_provider: PaymentProvider,
    pub onramper_address: String,
    pub proof_submitted: bool,
    pub payment_done: bool,
    pub locked_at: u64,
}

impl Storable for OrderState {
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
