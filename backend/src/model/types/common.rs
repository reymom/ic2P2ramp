use candid::{CandidType, Deserialize};
use std::{collections::HashMap, hash::Hash};

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

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum LoginAddress {
    Email { email: String, password: String },
    EVM { address: String },
    ICP { principal_id: String },
    Solana { address: String },
}

impl LoginAddress {
    pub fn validate(&self) -> Result<()> {
        match self {
            LoginAddress::Email { email, .. } => {
                if email.is_empty() {
                    return Err(RampError::InvalidInput("Email is empty".to_string()));
                }
                helpers::validate_email(email)?
            }
            LoginAddress::EVM { address } => {
                if address.is_empty() {
                    return Err(RampError::InvalidInput("EVM address is empty".to_string()));
                }
                helpers::validate_evm_address(address)?;
            }
            LoginAddress::ICP { principal_id } => {
                if principal_id.is_empty() {
                    return Err(RampError::InvalidInput(
                        "ICP principal ID is empty".to_string(),
                    ));
                }
                helpers::validate_icp_address(principal_id)?;
            }
            LoginAddress::Solana { address } => {
                if address.is_empty() {
                    return Err(RampError::InvalidInput(
                        "Solana address is empty".to_string(),
                    ));
                }
                helpers::validate_solana_address(address)?;
            }
        }
        Ok(())
    }

    pub fn to_transaction_address(&self) -> Result<TransactionAddress> {
        match self {
            LoginAddress::Email { .. } => Err(RampError::InvalidInput(
                "Cannot convert Email to TransactionAddress".to_string(),
            )),
            LoginAddress::EVM { address } => Ok(TransactionAddress {
                address_type: AddressType::EVM,
                address: address.clone(),
            }),
            LoginAddress::ICP { principal_id } => Ok(TransactionAddress {
                address_type: AddressType::ICP,
                address: principal_id.clone(),
            }),
            LoginAddress::Solana { address } => Ok(TransactionAddress {
                address_type: AddressType::Solana,
                address: address.clone(),
            }),
        }
    }
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq, Hash, PartialOrd)]
pub enum AddressType {
    EVM,
    ICP,
    Solana,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq)]
pub struct TransactionAddress {
    pub address_type: AddressType,
    pub address: String,
}

impl PartialEq for TransactionAddress {
    fn eq(&self, other: &Self) -> bool {
        self.address_type == other.address_type
    }
}

impl Hash for TransactionAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.address_type.hash(state);
    }
}

impl TransactionAddress {
    pub fn validate(&self) -> Result<()> {
        if self.address.is_empty() {
            return Err(RampError::InvalidInput("Address is empty".to_string()));
        }

        match self.address_type {
            AddressType::EVM => helpers::validate_evm_address(&self.address),
            AddressType::ICP => helpers::validate_icp_address(&self.address),
            AddressType::Solana => helpers::validate_solana_address(&self.address),
        }?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::model::types::common::{LoginAddress, TransactionAddress};
    use crate::model::types::user::User;
    use crate::types::{common::AddressType, user::UserType, PaymentProvider};

    use candid::Principal;
    use ethers_core::types::Address as EthAddress;
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
    use std::collections::HashSet;

    #[test]
    fn test_stable_btree_map() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<u64, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let mut payment_providers = HashSet::new();
        payment_providers.insert(PaymentProvider::PayPal {
            id: "paypal_id".to_string(),
        });

        let login_address = LoginAddress::EVM {
            address: (format!("{:#x}", EthAddress::random())),
        };

        let mut user = User::new(UserType::Offramper, login_address.clone()).unwrap();
        map.insert(0, user.clone());

        let retrieved_user = map.get(&0).unwrap();
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
        map.insert(updated_user.id, updated_user.clone());

        let retrieved_updated_user = map.get(&updated_user.id).unwrap();
        assert_eq!(
            updated_user.payment_providers,
            retrieved_updated_user.payment_providers
        );

        // Add address
        let new_address = TransactionAddress {
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

        map.insert(updated_user.id, updated_user.clone());

        let retrieved_user_with_new_address = map.get(&updated_user.id).unwrap();
        assert!(retrieved_user_with_new_address
            .addresses
            .contains(&new_address));
    }

    #[test]
    fn test_add_address() {
        let login_address = LoginAddress::EVM {
            address: (format!("{:#x}", EthAddress::random())),
        };

        let mut user = User::new(UserType::Offramper, login_address.clone()).unwrap();

        let new_address = TransactionAddress {
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
        let updated_address = TransactionAddress {
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
        let mut map: StableBTreeMap<u64, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let login_address1 = LoginAddress::EVM {
            address: (format!("{:#x}", EthAddress::random())),
        };
        let login_address2 = LoginAddress::EVM {
            address: (format!("{:#x}", EthAddress::random())),
        };

        let user1 = User::new(UserType::Offramper, login_address1.clone()).unwrap();
        let user2 = User::new(UserType::Onramper, login_address2.clone()).unwrap();

        map.insert(1, user1.clone());
        map.insert(2, user2.clone());

        let retrieved_user1 = map.get(&1).unwrap();
        let retrieved_user2 = map.get(&2).unwrap();

        assert_eq!(user1.addresses, retrieved_user1.addresses);
        assert_eq!(user2.addresses, retrieved_user2.addresses);
    }
}
