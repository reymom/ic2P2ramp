use candid::Principal;
use icramp_types::solana::errors::Result;
use sol_rpc_types::SolanaCluster;
use std::{cell::RefCell, collections::HashMap};

use crate::model::types::ed25519::Ed25519KeyName;

use super::state::State;

thread_local! {
    /// Heap‐state
    pub(crate) static STATE: RefCell<Option<State>> = RefCell::default();

    /// Registered SPL‐token mints → metadata (if you need to track tokens)
    pub(crate) static TOKENS: RefCell<HashMap<String, u8>> = RefCell::new(HashMap::new());
}

pub fn init_state(state: State) {
    STATE.with(|s| *s.borrow_mut() = Some(state));
}

pub fn get_state() -> State {
    STATE.with_borrow(|state| {
        state
            .as_ref()
            .expect("BUG: state is not initialized")
            .clone()
    })
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|s| f(s.borrow().as_ref().expect("STATE not initialised")))
}

pub fn mutate_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| {
        let mut mut_ref = s.borrow_mut();
        let st = mut_ref.as_mut().expect("STATE not initialised");
        f(st)
    })
}

pub fn network() -> SolanaCluster {
    read_state(|s| s.network)
}
pub fn key_name() -> Ed25519KeyName {
    read_state(|s| s.ed25519_key_name)
}
pub fn sol_rpc_canister() -> Principal {
    read_state(|s| s.sol_rpc_canister_id)
}
pub fn proxy_url() -> String {
    read_state(|s| s.proxy_url.clone())
}

pub(crate) fn get_tokens() -> HashMap<String, u8> {
    TOKENS.with(|token| token.borrow().clone())
}

pub(crate) fn set_tokens(runes: HashMap<String, u8>) {
    TOKENS.with_borrow_mut(|t| *t = runes)
}

/// Register SPL‐token mint
pub fn register_tokens(new_tokens: HashMap<String, u8>) -> Result<()> {
    TOKENS.with_borrow_mut(|tokens| {
        tokens.extend(new_tokens);
        Ok(())
    })
}

/// Is this mint known?
pub fn is_token_registered(mint: &str) -> bool {
    TOKENS.with(|m| m.borrow().contains_key(mint))
}
