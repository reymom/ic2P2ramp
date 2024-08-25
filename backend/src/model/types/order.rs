use candid::{CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::time;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashMap, fmt};

use crate::{
    errors::{RampError, Result},
    state,
};

use super::{
    blockchain::{Blockchain, Crypto},
    common::{calculate_fees, AddressType, PaymentProvider, PaymentProviderType},
    TransactionAddress,
};

const MAX_ORDER_SIZE: u32 = 8000;

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
    pub offramper_user_id: u64,
    pub created_at: u64,
    pub fiat_amount: u64,
    pub offramper_fee: u64,
    pub currency_symbol: String,
    pub offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    pub crypto: Crypto,
    pub offramper_address: TransactionAddress,
}

impl Order {
    pub fn new(
        offramper_user_id: u64,
        fiat_amount: u64,
        currency_symbol: String,
        offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
        blockchain: Blockchain,
        token: Option<String>,
        crypto_amount: u128,
        offramper_address: TransactionAddress,
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
            offramper_user_id,
            created_at: time(),
            fiat_amount,
            offramper_fee,
            currency_symbol,
            offramper_providers,
            crypto: Crypto::new(blockchain, token, crypto_amount, crypto_fee),
            offramper_address,
        };
        ic_cdk::println!("[new order] order = {:?}", order);

        Ok(order)
    }

    pub fn lock(
        self,
        onramper_user_id: u64,
        onramper_provider: PaymentProvider,
        onramper_address: TransactionAddress,
        consent_id: Option<String>,
        consent_url: Option<String>,
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
            onramper_user_id,
            onramper_address,
            onramper_provider,
            consent_id,
            consent_url,
            payment_done: false,
            payment_id: None,
            locked_at: time(),
        })
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct LockedOrder {
    pub base: Order,
    pub onramper_user_id: u64,
    pub onramper_provider: PaymentProvider,
    pub onramper_address: TransactionAddress,
    pub consent_id: Option<String>,
    pub consent_url: Option<String>,
    pub payment_done: bool,
    pub payment_id: Option<String>,
    pub locked_at: u64,
}

impl LockedOrder {
    pub fn complete(self) -> CompletedOrder {
        self.into()
    }
}

#[derive(CandidType, Deserialize, Clone)]
pub struct CompletedOrder {
    pub onramper: TransactionAddress,
    pub offramper: TransactionAddress,
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
