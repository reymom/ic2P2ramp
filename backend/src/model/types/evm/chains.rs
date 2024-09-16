use std::collections::HashMap;

use ethers_core::types::U256;

use crate::{
    errors::{RampError, Result},
    evm::rpc::RpcServices,
    model::memory::heap::{mutate_state, read_state},
};

use super::{gas::ChainGasTracking, token::Token};

#[derive(Clone, Debug)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub nonce: U256,
    pub approved_tokens: HashMap<String, Token>,
    pub gas_tracking: ChainGasTracking,
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

pub fn chain_is_supported(chain_id: u64) -> Result<()> {
    read_state(|state| {
        if state.chains.contains_key(&chain_id) {
            Ok(())
        } else {
            Err(RampError::ChainIdNotFound(chain_id))
        }
    })
}
