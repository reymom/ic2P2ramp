use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

use crate::errors::{RampError, Result};
use crate::model::types::LoginAddress;
use crate::types::{
    order::{Order, OrderId, OrderState},
    user::User,
};

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    pub static USERS: RefCell<StableBTreeMap<u64, User, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    pub static ORDERS: RefCell<StableBTreeMap<OrderId, OrderState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        )
    );
}

// -----
// USERS
// -----

pub fn mutate_user<F, R>(user_id: u64, f: F) -> Result<R>
where
    F: FnOnce(&mut User) -> R,
{
    USERS.with_borrow_mut(|users| {
        if let Some(mut user) = users.get(&user_id) {
            let result = f(&mut user);
            users.insert(user_id, user);
            Ok(result)
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn insert_user(user: &User) -> Option<User> {
    USERS.with_borrow_mut(|p| p.insert(user.id, user.clone()))
}

pub fn remove_user(user_id: &u64) -> Result<User> {
    USERS
        .with_borrow_mut(|p| p.remove(&user_id))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn get_user(user_id: &u64) -> Result<User> {
    USERS
        .with_borrow(|users| users.get(&user_id))
        .ok_or_else(|| RampError::UserNotFound)
}

pub fn find_user_by_login_address(login_address: &LoginAddress) -> Result<u64> {
    USERS.with(|users| {
        for (id, user) in users.borrow().iter() {
            if user.login == *login_address {
                return Ok(id);
            }
        }
        Err(RampError::UserNotFound)
    })
}

pub fn reset_password_user(login_address: &LoginAddress, password: String) -> Result<u64> {
    USERS.with_borrow_mut(|users| {
        let mut user_to_update = None;

        for (_, user) in users.iter() {
            if user.login == *login_address {
                user_to_update = Some(User {
                    hashed_password: Some(password.clone()),
                    ..user
                })
            }
        }

        if let Some(user) = user_to_update {
            let id = user.id;
            users.insert(id, user);
            return Ok(id);
        } else {
            return Err(RampError::UserNotFound);
        }
    })
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

pub fn filter_orders<F>(filter: F, page: Option<u32>, page_size: Option<u32>) -> Vec<OrderState>
where
    F: Fn(&OrderState) -> bool,
{
    let start_index = page.unwrap_or(1).saturating_sub(1) * page_size.unwrap_or(10);
    let end_index = start_index + page_size.unwrap_or(10);

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
            .skip(start_index as usize)
            .take((end_index - start_index) as usize)
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
