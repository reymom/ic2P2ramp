mod blockchain;
mod common;
pub mod evm;
pub mod exchange_rate;
pub mod icp;
pub mod orders;
pub mod payment;
pub mod session;
pub mod user;

pub use blockchain::{Blockchain, Crypto};
pub use common::{AddressType, AuthenticationData, LoginAddress, TransactionAddress};
pub use payment::providers::{contains_provider_type, PaymentProvider, PaymentProviderType};

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

        let mut user = User::new(UserType::Offramper, login_address.clone(), None).unwrap();
        map.insert(0, user.clone());

        let retrieved_user = map.get(&0).unwrap();
        assert_eq!(user.payment_providers, retrieved_user.payment_providers);
        assert_eq!(user.fiat_amounts, retrieved_user.fiat_amounts);
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

        let mut user = User::new(UserType::Offramper, login_address.clone(), None).unwrap();

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

        let user1 = User::new(UserType::Offramper, login_address1.clone(), None).unwrap();
        let user2 = User::new(UserType::Onramper, login_address2.clone(), None).unwrap();

        map.insert(1, user1.clone());
        map.insert(2, user2.clone());

        let retrieved_user1 = map.get(&1).unwrap();
        let retrieved_user2 = map.get(&2).unwrap();

        assert_eq!(user1.addresses, retrieved_user1.addresses);
        assert_eq!(user2.addresses, retrieved_user2.addresses);
    }
}
