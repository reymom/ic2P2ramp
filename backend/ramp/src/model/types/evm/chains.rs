use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use evm_rpc_canister_types::RpcServices;

use crate::{
    errors::{BlockchainError, Result, SystemError},
    model::memory::heap::read_state,
};

use super::{gas::ChainGasTracking, nonce::NonceManagement, token::Token};

/// Represents the state of a specific blockchain (or chain) including nonce management,
/// vault addresses, RPC services, and gas tracking.
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub currency_symbol: String,
    pub(super) nonce_manager: NonceManagement,
    pub approved_tokens: HashMap<String, Token>,
    pub gas_tracking: ChainGasTracking,
}

impl ChainState {
    /// Creates a new `ChainState` instance with default nonce and unlocked state.
    pub fn new(
        vault_manager_address: String,
        rpc_services: RpcServices,
        currency_symbol: String,
    ) -> Self {
        Self {
            vault_manager_address,
            rpc_services,
            currency_symbol,
            nonce_manager: NonceManagement::new(),
            approved_tokens: HashMap::new(),
            gas_tracking: ChainGasTracking::default(),
        }
    }
}

/// Retrieves the RPC services for a specific chain.
/// Returns the RPC services or an error if the chain ID is not found.
pub fn get_rpc_providers(chain_id: u64) -> Result<RpcServices> {
    read_state(|s| {
        s.chains
            .get(&chain_id)
            .map(|chain_state| chain_state.rpc_services.clone())
            .ok_or_else(|| BlockchainError::ChainIdNotFound(chain_id).into())
    })
}

/// Retrieves the vault manager address for a specific chain.
/// Returns the address or an error if the address or chain ID is not found.
pub fn get_vault_manager_address(chain_id: u64) -> Result<String> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| BlockchainError::ChainIdNotFound(chain_id).into())
            .and_then(|chain_state| {
                if chain_state.vault_manager_address.is_empty() {
                    Err(BlockchainError::VaultManagerAddressNotFound(chain_id).into())
                } else {
                    Ok(chain_state.vault_manager_address.clone())
                }
            })
    })
}

/// Retrieves the currency symbol of the native currency for a specific chain.
/// Returns the symbol or an error if the symbol or chain ID is not found.
pub fn get_native_currency_symbol(chain_id: u64) -> Result<String> {
    read_state(|state| {
        state
            .chains
            .get(&chain_id)
            .ok_or_else(|| BlockchainError::ChainIdNotFound(chain_id).into())
            .and_then(|chain_state| {
                if chain_state.currency_symbol.is_empty() {
                    Err(SystemError::CurrencySymbolNotFound().into())
                } else {
                    Ok(chain_state.currency_symbol.clone())
                }
            })
    })
}

/// Checks if a specific chain ID is supported (i.e., exists in the state).
/// Returns `Ok(())` if the chain is supported, or an error if the chain is not found.
pub fn chain_is_supported(chain_id: u64) -> Result<()> {
    read_state(|state| {
        if state.chains.contains_key(&chain_id) {
            Ok(())
        } else {
            Err(BlockchainError::ChainIdNotFound(chain_id).into())
        }
    })
}
