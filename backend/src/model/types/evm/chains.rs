use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ethers_core::types::U256;

use crate::{
    errors::{BlockchainError, Result, SystemError},
    evm::rpc::RpcServices,
    model::memory::heap::{mutate_state, read_state},
};

use super::{gas::ChainGasTracking, token::Token};

/// A constant defining the amount of time (in seconds) that a nonce can remain locked.
pub const LOCK_NONCE_TIME_SECONDS: u64 = 5;

/// Represents the state of a specific blockchain (or chain) including nonce management,
/// vault addresses, RPC services, and gas tracking.
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct ChainState {
    pub vault_manager_address: String,
    pub rpc_services: RpcServices,
    pub currency_symbol: String,
    nonce: u128,
    nonce_locked: bool,
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
            nonce: 0,
            nonce_locked: false,
            approved_tokens: HashMap::new(),
            gas_tracking: ChainGasTracking::default(),
        }
    }

    pub fn get_nonce_as_u256(&self) -> U256 {
        U256::from(self.nonce)
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Locks the nonce, preventing other transactions from using it.
    pub fn lock_nonce(&mut self) {
        self.nonce_locked = true;
    }

    /// Unlocks the nonce, making it available for future transactions.
    pub fn unlock_nonce(&mut self) {
        self.nonce_locked = false;
    }
}

/// Checks if the nonce for a specific chain is currently locked.
/// Returns `true` if the nonce is locked, `false` otherwise.
pub fn nonce_locked(chain_id: u64) -> bool {
    read_state(|state| {
        if let Some(chain_state) = state.chains.get(&chain_id) {
            ic_cdk::println!(
                "[nonce_locked] chain_state.nonce_locked = {}",
                chain_state.nonce_locked
            );
            chain_state.nonce_locked
        } else {
            false
        }
    })
}

/// Retrieves and locks the nonce for a specific chain, preventing other transactions from using it.
/// Returns the current nonce as a `U256` or an error if the chain ID is not found.
pub fn get_and_lock_nonce(chain_id: u64) -> Result<U256> {
    ic_cdk::println!("[get_and_lock_nonce] locking...");
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.lock_nonce();
            Ok(chain_state.get_nonce_as_u256())
        } else {
            Err(BlockchainError::ChainIdNotFound(chain_id).into())
        }
    })
}

/// Releases the nonce lock for a specific chain, allowing future transactions to use it.
pub fn release_nonce(chain_id: u64) {
    ic_cdk::println!("[release_nonce]");
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.unlock_nonce();
        }
    });
}

/// Releases the nonce lock and increments the nonce for a specific chain.
/// Optionally, the nonce can be explicitly set to `used_nonce` + 1 if provided.
pub fn release_and_increment_nonce(chain_id: u64, used_nonce: Option<u128>) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.unlock_nonce();

            ic_cdk::println!(
                "[release_and_increment_nonce] prev nonce: {}, new nonce: {}",
                chain_state.nonce,
                used_nonce.unwrap_or_else(|| chain_state.nonce) + 1
            );
            if let Some(nonce) = used_nonce {
                chain_state.nonce = nonce + 1;
            } else {
                chain_state.increment_nonce();
            }
        }
    });
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
pub fn get_currency_symbol(chain_id: u64) -> Result<String> {
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
