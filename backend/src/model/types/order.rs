use candid::{CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashMap, fmt};

use crate::{
    errors::{RampError, Result},
    model::memory,
};

use super::{
    blockchain::{Blockchain, Crypto},
    common::AddressType,
    PaymentProvider, PaymentProviderType, TransactionAddress,
};

const MAX_ORDER_SIZE: u32 = 8000;

pub(crate) const OFFRAMPER_FIAT_FEE_DENOM: u64 = 40; // 2.5%
pub(crate) const ADMIN_CRYPTO_FEE_DENOM: u128 = 200; // 0.5%

pub fn get_fiat_fee(fiat_amount: u64) -> u64 {
    fiat_amount / OFFRAMPER_FIAT_FEE_DENOM
}

pub fn get_crypto_fee(crypto_amount: u128, blockchain_fees: u128) -> u128 {
    let admin_fee = crypto_amount / ADMIN_CRYPTO_FEE_DENOM;
    blockchain_fees + admin_fee
}

pub type OrderId = u64;

#[derive(CandidType, Deserialize, Clone)]
pub enum OrderState {
    Created(Order),
    Locked(LockedOrder),
    Completed(CompletedOrder),
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

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Order {
    pub id: u64,
    pub created_at: u64,
    pub currency: String,
    pub offramper_user_id: u64,
    pub offramper_address: TransactionAddress,
    pub offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    pub crypto: Crypto,
}

impl Order {
    pub fn new(
        currency: String,
        offramper_user_id: u64,
        offramper_address: TransactionAddress,
        offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
        blockchain: Blockchain,
        token: Option<String>,
        crypto_amount: u128,
        crypto_fee: u128,
    ) -> Result<Self> {
        offramper_address.validate()?;

        match (blockchain.clone(), &offramper_address.address_type) {
            (Blockchain::EVM { .. }, AddressType::EVM)
            | (Blockchain::ICP { .. }, AddressType::ICP)
            | (Blockchain::Solana, AddressType::Solana) => (),
            _ => {
                return Err(RampError::InvalidInput(
                    "Address type does not match blockchain type".to_string(),
                ));
            }
        }

        let order_id = memory::heap::generate_order_id();
        let order = Order {
            id: order_id.clone(),
            currency,
            created_at: time(),
            offramper_user_id,
            offramper_address,
            offramper_providers,
            crypto: Crypto::new(blockchain, token, crypto_amount, crypto_fee),
        };
        ic_cdk::println!("[new order] order = {:?}", order);

        Ok(order)
    }

    pub fn lock(
        self,
        price: u64,
        offramper_fee: u64,
        onramper_user_id: u64,
        onramper_provider: PaymentProvider,
        onramper_address: TransactionAddress,
        revolut_consent: Option<RevolutConsent>,
    ) -> Result<LockedOrder> {
        // Check if the address type matches the blockchain type
        match (
            self.crypto.blockchain.clone(),
            &onramper_address.address_type,
        ) {
            (Blockchain::EVM { .. }, AddressType::EVM)
            | (Blockchain::ICP { .. }, AddressType::ICP)
            | (Blockchain::Solana, AddressType::Solana) => (),
            _ => {
                return Err(RampError::InvalidInput(
                    "Address type does not match blockchain type".to_string(),
                ));
            }
        }

        Ok(LockedOrder {
            base: self,
            locked_at: time(),
            price,
            offramper_fee,
            onramper: Onramper::new(onramper_user_id, onramper_provider, onramper_address),
            revolut_consent,
            payment_done: false,
            payment_id: None,
            uncommited: false,
        })
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Onramper {
    pub user_id: u64,
    pub provider: PaymentProvider,
    pub address: TransactionAddress,
}

impl Onramper {
    pub fn new(user_id: u64, provider: PaymentProvider, address: TransactionAddress) -> Self {
        Onramper {
            user_id,
            provider,
            address,
        }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct RevolutConsent {
    pub id: String,
    url: String,
}

impl RevolutConsent {
    pub fn new(id: String, url: String) -> Self {
        RevolutConsent { id, url }
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LockedOrder {
    pub base: Order,
    pub locked_at: u64,
    pub price: u64,
    pub offramper_fee: u64,
    pub onramper: Onramper,
    pub revolut_consent: Option<RevolutConsent>,
    pub payment_id: Option<String>,
    pub payment_done: bool,
    pub uncommited: bool,
}

impl LockedOrder {
    pub fn complete(self) -> CompletedOrder {
        self.into()
    }

    pub fn uncommit(&mut self) {
        self.uncommited = true;
    }

    pub fn payment_amount_matches(&self, received_amount: &str) -> bool {
        let total_expected_amount = (self.price + self.offramper_fee) as f64 / 100.0;
        let received_amount_as_float = received_amount.parse::<f64>().unwrap_or(0.0);

        (received_amount_as_float - total_expected_amount).abs() < f64::EPSILON
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CompletedOrder {
    pub onramper: TransactionAddress,
    pub offramper: TransactionAddress,
    pub price: u64,
    pub offramper_fee: u64,
    pub blockchain: Blockchain,
    pub completed_at: u64,
}

impl From<LockedOrder> for CompletedOrder {
    fn from(locked_order: LockedOrder) -> Self {
        let base = locked_order.base;
        CompletedOrder {
            onramper: locked_order.onramper.address,
            offramper: base.offramper_address,
            price: locked_order.price,
            offramper_fee: locked_order.offramper_fee,
            blockchain: base.crypto.blockchain,
            completed_at: time(),
        }
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderFilter {
    ByOfframperId(u64),
    ByOnramperId(u64),
    ByOfframperAddress(TransactionAddress),
    LockedByOnramper(TransactionAddress),
    ByState(OrderStateFilter),
    ByBlockchain(Blockchain),
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderStateFilter {
    Created,
    Locked,
    Completed,
    Cancelled,
}
