use ic_cdk_timers::{clear_timer, set_timer, TimerId};
use std::{cell::RefCell, collections::HashMap, time::Duration};

use super::State;
use crate::{
    management,
    model::{
        errors::{RampError, Result},
        types::evm::logs::EvmTransactionLog,
    },
};

const LOCK_DURATION_TIME_SECONDS: u64 = 120; // 1 hour

thread_local! {
    pub(crate) static STATE: RefCell<Option<State>> = RefCell::default();

    static USER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
    static ORDER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
    static LOCKED_ORDER_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::default();

    pub(crate) static EVM_TRANSACTION_LOGS: RefCell<HashMap<u64, EvmTransactionLog>> = RefCell::new(HashMap::new());
    pub(crate) static TRANSACTION_LOG_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::new(HashMap::new());
}

pub fn generate_order_id() -> u64 {
    ORDER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    })
}

pub fn generate_user_id() -> u64 {
    USER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    })
}

pub async fn set_order_timer(order_id: u64) {
    let duration = LOCK_DURATION_TIME_SECONDS;
    let timer_id = set_timer(Duration::from_secs(duration), move || {
        ic_cdk::spawn(async move {
            if let Err(e) = management::order::unlock_order(order_id, None).await {
                ic_cdk::println!("Failed to auto-unlock order {}: {:?}", order_id, e);
            }
        });
    });

    LOCKED_ORDER_TIMERS.with_borrow_mut(|timer| {
        timer.insert(order_id, timer_id);
    });
}

pub fn clear_order_timer(order_id: u64) -> Result<()> {
    LOCKED_ORDER_TIMERS.with_borrow_mut(|timer| match timer.remove(&order_id) {
        Some(timer_id) => {
            clear_timer(timer_id);
            Ok(())
        }
        None => Err(RampError::OrderTimerNotFound),
    })
}

// -----------
// For Upgrade
// -----------

pub(crate) fn get_user_id_counter() -> u64 {
    USER_ID_COUNTER.with(|counter| *counter.borrow())
}

pub(crate) fn set_user_id_counter(value: u64) {
    USER_ID_COUNTER.with(|counter| *counter.borrow_mut() = value);
}

pub(crate) fn get_order_id_counter() -> u64 {
    ORDER_ID_COUNTER.with(|counter| *counter.borrow())
}

pub(crate) fn set_order_id_counter(value: u64) {
    ORDER_ID_COUNTER.with(|counter| *counter.borrow_mut() = value);
}

pub(crate) fn get_locked_order_timers() -> HashMap<u64, TimerId> {
    LOCKED_ORDER_TIMERS.with(|timers| timers.borrow().clone())
}

pub(crate) fn set_locked_order_timers(timers: HashMap<u64, TimerId>) {
    LOCKED_ORDER_TIMERS.with(|map| *map.borrow_mut() = timers);
}
