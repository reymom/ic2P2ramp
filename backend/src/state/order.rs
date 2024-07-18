use candid::{CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashMap, fmt};

use crate::errors::{RampError, Result};

use super::{
    blockchain::{Blockchain, Crypto},
    common::{calculate_fees, AddressType, PaymentProvider, PaymentProviderType},
    state,
    storage::Address,
};

const MAX_ORDER_SIZE: u32 = 500;

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

#[derive(CandidType, Deserialize, Clone)]
pub struct Order {
    pub id: u64,
    pub created_at: u64,
    pub fiat_amount: u64,
    pub offramper_fee: u64,
    pub currency_symbol: String,
    pub offramper_providers: HashMap<PaymentProviderType, String>,
    pub crypto: Crypto,
    pub offramper_address: Address,
}

impl Order {
    pub fn new(
        fiat_amount: u64,
        currency_symbol: String,
        offramper_providers: HashMap<PaymentProviderType, String>,

        blockchain: Blockchain,
        token: Option<String>,
        crypto_amount: u64,

        offramper_address: Address,
    ) -> Result<Self> {
        offramper_address.validate()?;

        // Check if the address type matches the blockchain type
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

        let order_id = state::generate_order_id();
        let (offramper_fee, crypto_fee) = calculate_fees(fiat_amount, crypto_amount);

        let order = Order {
            id: order_id.clone(),
            created_at: time(),
            fiat_amount,
            offramper_fee,
            currency_symbol,
            offramper_providers,
            crypto: Crypto::new(blockchain, token, crypto_amount, crypto_fee),
            offramper_address,
        };

        Ok(order)
    }

    pub fn lock(
        self,
        onramper_provider: PaymentProvider,
        onramper_address: Address,
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
            onramper_address,
            onramper_provider,
            payment_done: false,
            locked_at: time(),
        })
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LockedOrder {
    pub base: Order,
    pub onramper_provider: PaymentProvider,
    pub onramper_address: Address,
    pub payment_done: bool,
    pub locked_at: u64,
}

impl LockedOrder {
    pub fn complete(self) -> CompletedOrder {
        self.into()
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CompletedOrder {
    pub onramper: Address,
    pub offramper: Address,
    pub fiat_amount: u64,
    pub offramper_fee: u64,
    pub blockchain: Blockchain,
    pub completed_at: u64,
}

impl From<LockedOrder> for CompletedOrder {
    fn from(locked_order: LockedOrder) -> Self {
        let base = locked_order.base;
        CompletedOrder {
            onramper: locked_order.onramper_address,
            offramper: base.offramper_address,
            fiat_amount: base.fiat_amount,
            offramper_fee: base.offramper_fee,
            blockchain: base.crypto.blockchain,
            completed_at: time(),
        }
    }
}

#[derive(CandidType, Clone, Deserialize)]
pub enum OrderFilter {
    ByOfframperAddress(Address),
    LockedByOnramper(Address),
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
