use ic_cdk_timers::{clear_timer, set_timer, TimerId};
use std::{cell::RefCell, collections::HashMap, time::Duration};

use super::State;
use crate::{
    errors::{OrderError, Result},
    management,
    types::{evm::logs::EvmTransactionLog, exchange_rate::ExchangeRateCache},
};

pub(crate) const LOCK_DURATION_TIME_SECONDS: u64 = 1800; // 30 min

thread_local! {
    pub(crate) static STATE: RefCell<Option<State>> = RefCell::default();

    static USER_ID_COUNTER: RefCell<u64> = const { RefCell::new(0) };
    static ORDER_ID_COUNTER: RefCell<u64> = const { RefCell::new(0) };
    static LOCKED_ORDER_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::default();

    pub(super) static EVM_TRANSACTION_LOGS: RefCell<HashMap<u64, EvmTransactionLog>> = RefCell::new(HashMap::new());
    pub(super) static TRANSACTION_LOG_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::new(HashMap::new());
    pub(super) static EXCHANGE_RATE_CACHE: RefCell<HashMap<(String, String), ExchangeRateCache>> = RefCell::new(HashMap::new());
}

pub fn tmp_get_rate() -> HashMap<(String, String), ExchangeRateCache> {
    EXCHANGE_RATE_CACHE.with_borrow(|logs| logs.clone())
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

pub fn set_order_timer(order_id: u64) {
    let timer_id = set_timer(Duration::from_secs(LOCK_DURATION_TIME_SECONDS), move || {
        ic_cdk::spawn(async move {
            if let Err(e) = management::order::unlock_order(order_id).await {
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
        None => Err(OrderError::OrderTimerNotFound.into()),
    })
}

// -----------
// For Upgrade
// -----------

pub(super) fn get_user_id_counter() -> u64 {
    USER_ID_COUNTER.with(|counter| *counter.borrow())
}

pub(super) fn set_user_id_counter(value: u64) {
    USER_ID_COUNTER.with(|counter| *counter.borrow_mut() = value);
}

pub(super) fn get_order_id_counter() -> u64 {
    ORDER_ID_COUNTER.with(|counter| *counter.borrow())
}

pub(super) fn set_order_id_counter(value: u64) {
    ORDER_ID_COUNTER.with(|counter| *counter.borrow_mut() = value);
}

pub(super) fn get_locked_order_timers() -> HashMap<u64, TimerId> {
    LOCKED_ORDER_TIMERS.with(|timers| timers.borrow().clone())
}

pub(super) fn get_exchange_rate_cache() -> HashMap<(String, String), ExchangeRateCache> {
    EXCHANGE_RATE_CACHE.with_borrow(|logs| logs.clone())
}

pub(super) fn set_exchange_rate_cache(rates: HashMap<(String, String), ExchangeRateCache>) {
    EXCHANGE_RATE_CACHE.with_borrow_mut(|r| *r = rates);
}
