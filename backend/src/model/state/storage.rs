use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

use crate::errors::{RampError, Result};
use crate::types::{
    order::{Order, OrderState},
    user::User,
    Address,
};

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
