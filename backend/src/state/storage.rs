use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

pub use super::common::{Address, PaymentProvider, PaymentProviderType};
pub use super::order::{Order, OrderFilter, OrderState, OrderStateFilter};
pub use super::user::{User, UserType};
use crate::errors::{RampError, Result};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static USERS: RefCell<StableBTreeMap<Address, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ORDERS: RefCell<StableBTreeMap<u64, OrderState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );
}

// -----
// USERS
// -----

pub fn mutate_user<F, R>(address: &Address, f: F) -> Result<R>
where
    F: FnOnce(&mut User) -> R,
{
    USERS.with_borrow_mut(|users| {
        if let Some(mut user) = users.get(&address) {
            let result = f(&mut user);
            users.insert(address.clone(), user);
            Ok(result)
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn insert_user(user: &User) -> Option<User> {
    USERS.with_borrow_mut(|p| p.insert(user.login_method.clone(), user.clone()))
}

pub fn remove_user(address: &Address) -> Result<User> {
    USERS
        .with_borrow_mut(|p| p.remove(&address))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn get_user(address: &Address) -> Result<User> {
    USERS
        .with_borrow(|users| users.get(&address))
        .ok_or_else(|| RampError::UserNotFound)
}

// ------
// ORDERS
// ------

pub fn insert_order(order: &Order) -> Option<OrderState> {
    ORDERS.with_borrow_mut(|p| p.insert(order.id.clone(), OrderState::Created(order.clone())))
}

pub fn get_order(order_id: &u64) -> Result<OrderState> {
    ORDERS
        .with_borrow(|orders| orders.get(order_id))
        .ok_or_else(|| RampError::OrderNotFound)
}

pub fn filter_orders<F>(filter: F) -> Vec<OrderState>
where
    F: Fn(&OrderState) -> bool,
{
    ORDERS.with_borrow(|orders| {
        orders
            .iter()
            .filter_map(|(_, order_state)| {
                if filter(&order_state) {
                    Some(order_state.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

pub fn mutate_order<F, R>(order_id: &u64, f: F) -> Result<R>
where
    F: FnOnce(&mut OrderState) -> R,
{
    ORDERS.with_borrow_mut(|orders| {
        if let Some(mut order_state) = orders.get(&order_id) {
            let result = f(&mut order_state);
            orders.insert(*order_id, order_state);
            Ok(result)
        } else {
            Err(RampError::OrderNotFound)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::state::common::{AddressType, PaymentProviderType};
    use crate::state::user::UserType;

    use super::*;
    use candid::Principal;
    use ethers_core::types::Address as EthAddress;
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::DefaultMemoryImpl;
    use std::collections::HashSet;

    #[test]
    fn test_stable_btree_map() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<Address, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let mut payment_providers = HashSet::new();
        payment_providers.insert(PaymentProvider {
            provider_type: PaymentProviderType::PayPal,
            id: "paypal_id".to_string(),
        });

        let login_address = Address {
            address_type: AddressType::EVM,
            address: format!("{:#x}", EthAddress::random()),
        };

        let mut user = User::new(UserType::Offramper, login_address.clone()).unwrap();
        map.insert(login_address.clone(), user.clone());

        let retrieved_user = map.get(&login_address).unwrap();
        assert_eq!(user.payment_providers, retrieved_user.payment_providers);
        assert_eq!(user.fiat_amount, retrieved_user.fiat_amount);
        assert_eq!(user.score, retrieved_user.score);

        // Update user
        let mut updated_user = retrieved_user.clone();
        updated_user.payment_providers.insert(PaymentProvider {
            provider_type: PaymentProviderType::Revolut,
            id: "revolut_id".to_string(),
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

        let mut user = User::new(UserType::Offramper, login_address.clone()).unwrap();

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

        let user1 = User::new(UserType::Offramper, login_address1.clone()).unwrap();
        let user2 = User::new(UserType::Onramper, login_address2.clone()).unwrap();

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
