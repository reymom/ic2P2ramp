use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use ic_cdk_timers::{clear_timer, set_timer, TimerId};
use std::{cell::RefCell, collections::HashMap, time::Duration};

use crate::{
    errors::{RampError, Result},
    management,
};

use crate::types::{paypal::PayPalState, revolut::RevolutState, ChainState};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();

    static ORDER_ID_COUNTER: RefCell<u64> = RefCell::new(0);

    static LOCKED_ORDER_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::default();
}

#[derive(Clone, Debug)]
pub struct State {
    pub chains: HashMap<u64, ChainState>,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub paypal: PayPalState,
    pub revolut: RevolutState,
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidStateError {
    InvalidEthereumContractAddress(String),
}

/// Mutates (part of) the current state using `f`.
///
/// Panics if there is no state.
pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with_borrow_mut(|s| f(s.as_mut().expect("BUG: state is not initialized")))
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with_borrow(|s| f(s.as_ref().expect("BUG: state is not initialized")))
}

pub fn initialize_state(state: State) {
    STATE.set(Some(state));
}

pub fn increment_nonce(chain_id: u64) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.nonce += U256::from(1);
        }
    });
}

pub fn generate_order_id() -> u64 {
    ORDER_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        *counter += 1;
        *counter
    })
}

pub fn set_order_timer(order_id: u64) {
    let duration = 3600; // 1 hour
    let timer_id = set_timer(Duration::from_secs(duration), move || {
        ic_cdk::spawn(async move {
            if let Err(e) = management::order::unlock_order(order_id) {
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
