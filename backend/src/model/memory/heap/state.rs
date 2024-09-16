use std::collections::HashMap;

use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::model::types::{
    evm::chains::ChainState,
    payment::{paypal::PayPalState, revolut::RevolutState},
};

use super::heap::STATE;

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
