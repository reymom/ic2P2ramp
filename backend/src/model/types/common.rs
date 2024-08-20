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
pub enum PaymentProvider {
    PayPal {
        id: String,
    },
    Revolut {
        scheme: String,
        id: String,
        name: Option<String>,
    },
}

impl PartialEq for PaymentProvider {
    fn eq(&self, other: &Self) -> bool {
        self.provider_type() == other.provider_type()
    }
}

impl PaymentProvider {
    pub fn provider_type(&self) -> PaymentProviderType {
        match self {
            PaymentProvider::PayPal { .. } => PaymentProviderType::PayPal,
            PaymentProvider::Revolut { .. } => PaymentProviderType::Revolut,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            PaymentProvider::PayPal { id } => {
                if id.is_empty() {
                    return Err(RampError::InvalidInput("Paypal ID is empty".to_string()));
                }
            }
            PaymentProvider::Revolut { scheme, id, .. } => {
                if scheme.is_empty() || id.is_empty() {
                    return Err(RampError::InvalidInput(
                        "Revolut details are empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn contains_provider_type(
    provider: &PaymentProvider,
    providers: &HashMap<PaymentProviderType, PaymentProvider>,
) -> bool {
    providers.get(&provider.provider_type()).is_some()
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
const MAX_ADDRESS_SIZE: u32 = 200;

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

#[cfg(test)]
mod tests {
    use crate::model::types::user::User;
    use crate::types::{common::AddressType, user::UserType, Address, PaymentProvider};

    use candid::Principal;
    use ethers_core::types::Address as EthAddress;
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
    use std::collections::HashSet;

    #[test]
    fn test_stable_btree_map() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<Address, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let mut payment_providers = HashSet::new();
        payment_providers.insert(PaymentProvider::PayPal {
            id: "paypal_id".to_string(),
        });

        let login_address = Address {
            address_type: AddressType::EVM,
            address: format!("{:#x}", EthAddress::random()),
        };

        let mut user = User::new(UserType::Offramper, login_address.clone(), None).unwrap();
        map.insert(login_address.clone(), user.clone());

        let retrieved_user = map.get(&login_address).unwrap();
        assert_eq!(user.payment_providers, retrieved_user.payment_providers);
        assert_eq!(user.fiat_amount, retrieved_user.fiat_amount);
        assert_eq!(user.score, retrieved_user.score);

        // Update user
        let mut updated_user = retrieved_user.clone();
        updated_user
            .payment_providers
            .insert(PaymentProvider::Revolut {
                id: "revolut_id".to_string(),
                scheme: "scheme".to_string(),
                name: Some("name".to_string()),
            });
        map.insert(updated_user.login_method.clone(), updated_user.clone());

        let retrieved_updated_user = map.get(&login_address).unwrap();
        assert_eq!(
            updated_user.payment_providers,
            retrieved_updated_user.payment_providers
        );

        // Add address
        let new_address = Address {
            address_type: AddressType::ICP,
            address: Principal::anonymous().to_string(),
        };
        if let Some(existing_address) = user.addresses.take(&new_address) {
            ic_cdk::println!(
                "updating address {:?} to {:?}",
                existing_address,
                new_address
            )
        }
        updated_user.addresses.insert(new_address.clone());

        map.insert(updated_user.login_method.clone(), updated_user.clone());

        let retrieved_user_with_new_address = map.get(&login_address).unwrap();
        assert!(retrieved_user_with_new_address
            .addresses
            .contains(&new_address));
    }

    #[test]
    fn test_add_address() {
        let login_address = Address {
            address_type: AddressType::EVM,
            address: format!("{:#x}", EthAddress::random()),
        };

        let mut user = User::new(UserType::Offramper, login_address.clone(), None).unwrap();

        let new_address = Address {
            address_type: AddressType::ICP,
            address: "2chl6-4hpzw-vqaaa-aaaaa-c".to_string(),
        };
        if let Some(existing_address) = user.addresses.take(&new_address) {
            ic_cdk::println!(
                "updating address {:?} to {:?}",
                existing_address,
                new_address
            )
        }
        user.addresses.insert(new_address.clone());
        assert!(user.addresses.contains(&new_address));

        // Attempt to add the same address type should replace the old one
        let updated_address = Address {
            address_type: AddressType::ICP,
            address: Principal::anonymous().to_string(),
        };

        if let Some(existing_address) = user.addresses.take(&updated_address) {
            ic_cdk::println!(
                "updating address {:?} to {:?}",
                existing_address,
                new_address
            )
        }
        user.addresses.insert(updated_address.clone());
        assert_eq!(user.addresses.len(), 2);
        assert_eq!(
            user.addresses.get(&new_address).unwrap().address,
            updated_address.address
        );
    }

    #[test]
    fn test_multiple_users_same_address_type() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<Address, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let login_address1 = Address {
            address_type: AddressType::EVM,
            address: format!("{:#x}", EthAddress::random()),
        };
        let login_address2 = Address {
            address_type: AddressType::EVM,
            address: format!("{:#x}", EthAddress::random()),
        };

        let user1 = User::new(UserType::Offramper, login_address1.clone(), None).unwrap();
        let user2 = User::new(UserType::Onramper, login_address2.clone(), None).unwrap();

        map.insert(login_address1.clone(), user1.clone());
        map.insert(login_address2.clone(), user2.clone());

        let retrieved_user1 = map.get(&login_address1).unwrap();
        let retrieved_user2 = map.get(&login_address2).unwrap();

        assert_eq!(user1.addresses, retrieved_user1.addresses);
        assert_eq!(user2.addresses, retrieved_user2.addresses);
        assert_ne!(
            retrieved_user1
                .addresses
                .get(&login_address1)
                .unwrap()
                .address,
            retrieved_user2
                .addresses
                .get(&login_address2)
                .unwrap()
                .address,
        )
    }
}
