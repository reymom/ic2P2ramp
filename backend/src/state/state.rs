use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::{cell::RefCell, collections::HashMap};

use crate::{
    errors::{RampError, Result},
    evm::rpc::RpcServices,
};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Clone, Debug)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub nonce: U256,
    pub approved_tokens: HashMap<String, bool>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub chains: HashMap<u64, ChainState>,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub client_id: String,
    pub client_secret: String,
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

pub fn get_rpc_providers(chain_id: u64) -> RpcServices {
    read_state(|s| {
        s.chains
            .get(&chain_id)
            .map(|chain_state| chain_state.rpc_services.clone())
            .ok_or("Unsupported chain ID")
    })
    .unwrap()
}

pub fn get_vault_manager_address(chain_id: u64) -> Result<String> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))
            .and_then(|chain_state| {
                if chain_state.vault_manager_address.is_empty() {
                    Err(RampError::VaultManagerAddressNotFound(chain_id))
                } else {
                    Ok(chain_state.vault_manager_address.clone())
                }
            })
    })
}

pub fn token_is_approved(chain_id: u64, token_address: String) -> Result<bool> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))
            .map(|chain_state| {
                chain_state
                    .approved_tokens
                    .get(&token_address)
                    .cloned()
                    .unwrap_or(false)
            })
    })
}
