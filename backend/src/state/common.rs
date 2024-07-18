use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, cmp::Ordering, collections::HashMap, hash::Hash};

use crate::{
    errors::{RampError, Result},
    evm::helpers,
};

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PaymentProviderType {
    PayPal,
    Revolut,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, Hash)]
pub struct PaymentProvider {
    pub provider_type: PaymentProviderType,
    pub id: String,
}

impl PartialEq for PaymentProvider {
    fn eq(&self, other: &Self) -> bool {
        self.provider_type == other.provider_type
    }
}

impl PaymentProvider {
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(RampError::InvalidInput(
                "Payment Provider ID is empty".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn contains_provider_type(
    provider: &PaymentProvider,
    providers: &HashMap<PaymentProviderType, String>,
) -> bool {
    providers.get(&provider.provider_type).is_some()
}

pub fn calculate_fees(fiat_amount: u64, crypto_amount: u64) -> (u64, u64) {
    // Static strategy: 2% fee for the offramper, 0.5% for the admin
    let offramper_fee = fiat_amount / 50; // 2%
    let crypto_fee = crypto_amount / 200; // 0.5%

    (offramper_fee, crypto_fee)
}

// ---------
// Addresses
// ---------
const MAX_ADDRESS_SIZE: u32 = 100;

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub enum AddressType {
    Email,
    EVM,
    ICP,
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialOrd)]
pub struct Address {
    pub address_type: AddressType,
    pub address: String,
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        self.address_type == other.address_type
    }
}

impl Hash for Address {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address_type.hash(state);
    }
}

impl Address {
    pub fn validate(&self) -> Result<()> {
        if self.address.is_empty() {
            return Err(RampError::InvalidInput("Address is empty".to_string()));
        }

        match self.address_type {
            AddressType::EVM => helpers::validate_evm_address(&self.address),
            AddressType::ICP => helpers::validate_icp_address(&self.address),
            AddressType::Email => helpers::validate_email(&self.address),
            AddressType::Solana => helpers::validate_solana_address(&self.address),
        }?;

        Ok(())
    }
}

impl Storable for Address {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_ADDRESS_SIZE,
        is_fixed_size: false,
    };
}

impl std::cmp::Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}
