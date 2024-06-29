use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

use crate::errors::{RampError, Result};
use crate::management::validate_evm_address;

pub use super::common::PaymentProvider;
pub use super::order::{Order, OrderState};
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

    pub static ORDERS: RefCell<StableBTreeMap<String, OrderState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );

    static ORDER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
}

pub fn insert_user(user: &User) -> String {
    let evm_address = user.evm_address.clone();

    USERS.with_borrow_mut(|p| {
        p.insert(evm_address.clone(), user.clone());
    });
    evm_address
}

pub fn get_user(evm_address: &str) -> Result<User> {
    validate_evm_address(&evm_address)?;

    USERS
        .with_borrow(|users| users.get(&evm_address.to_string()))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn generate_order_id() -> String {
    ORDER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        counter.to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::DefaultMemoryImpl;

    #[test]
    fn test_stable_btree_map() {
        let memory_manager = MemoryManager::init(DefaultMemoryImpl::default());
        let mut map: StableBTreeMap<String, User, _> =
            StableBTreeMap::init(memory_manager.get(MemoryId::new(0)));

        let user = User {
            evm_address: "0x123".to_string(),
            payment_providers: vec![PaymentProvider::PayPal {
                id: "paypal_id".to_string(),
            }],
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
            .push(PaymentProvider::Revolut {
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
