use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

use crate::errors::{RampError, Result};
use crate::management::validate_evm_address;

pub use super::common::PaymentProvider;
pub use super::order::{Order, OrderFilter, OrderState, OrderStateFilter};
pub use super::user::User;

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static USERS: RefCell<StableBTreeMap<String, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ORDERS: RefCell<StableBTreeMap<u64, OrderState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static ORDER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
}

pub fn mutate_user<F, R>(evm_address: &str, f: F) -> Result<R>
where
    F: FnOnce(&mut User) -> R,
{
    USERS.with_borrow_mut(|users| {
        if let Some(mut user) = users.get(&evm_address.to_string()) {
            let result = f(&mut user);
            users.insert(evm_address.to_string(), user);
            Ok(result)
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn insert_user(user: &User) -> Option<User> {
    USERS.with_borrow_mut(|p| p.insert(user.evm_address.clone(), user.clone()))
}

pub fn remove_user(evm_address: &str) -> Result<User> {
    USERS
        .with_borrow_mut(|p| p.remove(&evm_address.to_string()))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn get_user(evm_address: &str) -> Result<User> {
    validate_evm_address(&evm_address)?;

    USERS
        .with_borrow(|users| users.get(&evm_address.to_string()))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn generate_order_id() -> u64 {
    ORDER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    })
}

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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::DefaultMemoryImpl;

    #[test]
    fn test_stable_btree_map() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<String, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let mut payment_providers = HashSet::new();
        payment_providers.insert(PaymentProvider::PayPal {
            id: "paypal_id".to_string(),
        });
        let user = User {
            evm_address: "0x123".to_string(),
            payment_providers,
            offramped_amount: 0,
            score: 1,
        };

        map.insert(user.evm_address.clone(), user.clone());
        let retrieved_user = map.get(&"0x123".to_string()).unwrap();

        assert_eq!(user.evm_address, retrieved_user.evm_address);
        assert_eq!(user.payment_providers, retrieved_user.payment_providers);
        assert_eq!(user.offramped_amount, retrieved_user.offramped_amount);
        assert_eq!(user.score, retrieved_user.score);

        // Update user
        let mut updated_user = retrieved_user.clone();
        updated_user
            .payment_providers
            .insert(PaymentProvider::Revolut {
                id: "revolut_id".to_string(),
            });
        map.insert(updated_user.evm_address.clone(), updated_user.clone());

        let retrieved_updated_user = map.get(&"0x123".to_string()).unwrap();
        assert_eq!(
            updated_user.payment_providers,
            retrieved_updated_user.payment_providers
        );
    }
}
