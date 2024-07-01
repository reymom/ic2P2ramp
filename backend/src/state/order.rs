use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashSet, fmt};

use crate::{errors::Result, evm::helpers};

use super::{common::PaymentProvider, storage};

const MAX_ORDER_SIZE: u32 = 500;

#[derive(CandidType, Deserialize, Clone)]
pub enum OrderState {
    Created(Order),
    Locked(LockedOrder),
    Completed(u64),
    Cancelled(u64),
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
    pub id: u64,
    pub originator: Principal,
    pub created_at: u64,
    pub fiat_amount: u64,
    pub currency_symbol: String,
    pub crypto_amount: u64,
    pub offramper_providers: HashSet<PaymentProvider>,
    pub offramper_address: String,
    pub chain_id: u64,
    pub token_address: Option<String>,
}

impl Order {
    pub fn new(
        fiat_amount: u64,
        currency_symbol: String,
        crypto_amount: u64,
        offramper_providers: HashSet<PaymentProvider>,
        offramper_address: String,
        chain_id: u64,
        token_address: Option<String>,
    ) -> Result<Self> {
        helpers::validate_evm_address(&offramper_address)?;

        let order_id = storage::generate_order_id();
        let order = Order {
            id: order_id.clone(),
            originator: ic_cdk::caller(),
            created_at: time(),
            fiat_amount,
            currency_symbol,
            crypto_amount,
            offramper_providers,
            offramper_address,
            chain_id,
            token_address,
        };

        Ok(order)
    }

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

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderFilter {
    ByOfframperAddress(String),
    LockedByOnramper(String),
    ByState(OrderStateFilter),
    ByChainId(u64),
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderStateFilter {
    Created,
    Locked,
    Completed,
    Cancelled,
}
