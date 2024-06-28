use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::{cell::RefCell, collections::HashMap};

use super::{chains::ChainState, paypal::PayPalState};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Clone, Debug)]
pub struct State {
    pub chains: HashMap<u64, ChainState>,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub paypal: PayPalState,
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
