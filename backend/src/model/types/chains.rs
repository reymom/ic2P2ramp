use std::collections::HashMap;

use ethers_core::types::U256;

use crate::{
    errors::{RampError, Result},
    evm::rpc::RpcServices,
    model::state::mutate_state,
    state::read_state,
};

use super::gas::{ChainGasTracking, MethodGasUsage};

#[derive(Clone, Debug)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub nonce: U256,
    pub approved_tokens: HashMap<String, String>, // <address, xrc symbol>
    pub gas_tracking: ChainGasTracking,
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

pub fn approve_evm_token(chain_id: u64, token_address: &str, xrc_symbol: &str) -> () {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state
                .approved_tokens
                .insert(token_address.to_string(), xrc_symbol.to_string());
        }
    })
}

pub fn evm_token_is_approved(chain_id: u64, token_address: &str) -> Result<()> {
    get_evm_token_symbol(chain_id, token_address).map(|_| ())
}

pub fn get_evm_token_symbol(chain_id: u64, token_address: &str) -> Result<String> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))
            .and_then(|chain_state| {
                chain_state
                    .approved_tokens
                    .get(token_address)
                    .ok_or_else(|| RampError::UnregisteredEvmToken)
                    .map(|symbol| symbol.clone())
            })
    })
}

pub fn register_gas_usage(
    chain_id: u64,
    gas: u128,
    gas_price: u128,
    block_number: u128,
    action_type: &MethodGasUsage,
) -> Result<()> {
    mutate_state(|state| {
        let chain_state = state
            .chains
            .get_mut(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))?;

        let gas_tracking = match action_type {
            MethodGasUsage::Commit => &mut chain_state.gas_tracking.commit_gas,
            MethodGasUsage::ReleaseToken => &mut chain_state.gas_tracking.release_token_gas,
            MethodGasUsage::ReleaseNative => &mut chain_state.gas_tracking.release_native_gas,
        };

        gas_tracking.record_gas_usage(gas, gas_price, block_number);

        Ok(())
    })
}

pub fn get_average_gas(
    chain_id: u64,
    current_block: u128,
    max_blocks_in_past: u64,
    action_type: &MethodGasUsage,
) -> Result<Option<(u128, u128)>> {
    read_state(|state| {
        let chain_state = state
            .chains
            .get(&chain_id)
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))?;

        let gas_tracking = match action_type {
            MethodGasUsage::Commit => &chain_state.gas_tracking.commit_gas,
            MethodGasUsage::ReleaseToken => &chain_state.gas_tracking.release_token_gas,
            MethodGasUsage::ReleaseNative => &chain_state.gas_tracking.release_native_gas,
        };

        Ok(gas_tracking.average_gas(current_block, max_blocks_in_past))
    })
}
