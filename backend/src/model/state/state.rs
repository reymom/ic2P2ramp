use candid::Principal;
use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use ic_cdk_timers::{clear_timer, set_timer, TimerId};
use icrc_ledger_types::icrc1::transfer::NumTokens;
use std::{cell::RefCell, collections::HashMap, time::Duration};

use crate::{
    errors::{RampError, Result},
    management,
};

use crate::types::{paypal::PayPalState, revolut::RevolutState, ChainState};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();

    static USER_ID_COUNTER: RefCell<u64> = RefCell::new(0);

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
    pub proxy_url: String,
    pub icp_fees: HashMap<Principal, NumTokens>,
    pub frontend_canister: Option<Principal>,
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

pub fn generate_user_id() -> u64 {
    USER_ID_COUNTER.with(|counter| {
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

pub fn get_fee(ledger_principal: &Principal) -> Result<NumTokens> {
    read_state(|state| {
        state
            .icp_fees
            .get(ledger_principal)
            .cloned()
            .ok_or_else(|| RampError::LedgerPrincipalNotSupported(ledger_principal.to_string()))
    })
}

pub fn set_frontend_canister(canister: &Principal) -> Result<()> {
    Ok(mutate_state(|state| {
        state.frontend_canister = Some(*canister);
    }))
}

pub fn is_token_supported(ledger_principal: &Principal) -> Result<()> {
    read_state(|state| {
        if state.icp_fees.contains_key(ledger_principal) {
            Ok(())
        } else {
            Err(RampError::LedgerPrincipalNotSupported(
                ledger_principal.to_string(),
            ))
        }
    })
}

pub fn is_chain_supported(chain_id: u64) -> Result<()> {
    read_state(|state| {
        if state.chains.contains_key(&chain_id) {
            Ok(())
        } else {
            Err(RampError::ChainIdNotFound(chain_id))
        }
    })
}

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
