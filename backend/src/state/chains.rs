use std::collections::HashMap;

use ethers_core::types::U256;

use crate::{
    errors::{RampError, Result},
    evm::rpc::RpcServices,
};

use super::read_state;

#[derive(Clone, Debug)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub nonce: U256,
    pub approved_tokens: HashMap<String, bool>,
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
