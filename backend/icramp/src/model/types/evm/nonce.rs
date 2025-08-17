use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ethers_core::types::U256;

use crate::{
    evm::fees::FeeEstimates,
    model::{
        errors::{BlockchainError, Result},
        memory::heap::{mutate_state, read_state},
    },
};

/// A constant defining the amount of time (in seconds) that a nonce can remain locked.
pub const LOCK_NONCE_TIME_SECONDS: u64 = 5;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct NonceManagement {
    current_nonce: u128,
    is_locked: bool,
    unresolved_nonces: HashMap<u128, NonceFeeEstimates>,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct NonceFeeEstimates {
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
}

impl From<NonceFeeEstimates> for FeeEstimates {
    fn from(fees: NonceFeeEstimates) -> FeeEstimates {
        FeeEstimates {
            max_fee_per_gas: fees.max_fee_per_gas.into(),
            max_priority_fee_per_gas: fees.max_priority_fee_per_gas.into(),
        }
    }
}

impl From<FeeEstimates> for NonceFeeEstimates {
    fn from(fees: FeeEstimates) -> NonceFeeEstimates {
        NonceFeeEstimates {
            max_fee_per_gas: fees.max_fee_per_gas.as_u128(),
            max_priority_fee_per_gas: fees.max_priority_fee_per_gas.as_u128(),
        }
    }
}

impl NonceManagement {
    pub(super) fn new() -> Self {
        Self {
            current_nonce: 0,
            is_locked: false,
            unresolved_nonces: HashMap::new(),
        }
    }

    pub fn get_nonce_as_u256(&self) -> U256 {
        U256::from(self.current_nonce)
    }

    /// Locks the nonce, preventing other transactions from using it.
    pub fn lock_nonce(&mut self) {
        self.is_locked = true;
    }

    /// Unlocks the nonce, making it available for future transactions.
    pub fn unlock_nonce(&mut self) {
        self.is_locked = false;
    }

    pub fn track_unresolved_nonce(&mut self, nonce: u128, fees: NonceFeeEstimates) {
        self.unresolved_nonces.insert(nonce, fees);
    }

    /// Checks if the nonce is unresolved and returns its associated `FeeEstimates`
    pub fn get_unresolved_fee_estimates(&self, nonce: u128) -> Option<NonceFeeEstimates> {
        self.unresolved_nonces.get(&nonce).cloned()
    }

    /// Clears all unresolved nonces (optional for cleanup)
    pub fn _clear_unresolved_nonces(&mut self) {
        self.unresolved_nonces.clear();
    }
}

/// Checks if the nonce for a specific chain is currently locked.
/// Returns `true` if the nonce is locked, `false` otherwise.
pub fn is_locked(chain_id: u64) -> bool {
    read_state(|state| {
        if let Some(chain_state) = state.chains.get(&chain_id) {
            chain_state.nonce_manager.is_locked
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
            chain_state.nonce_manager.lock_nonce();
            Ok(chain_state.nonce_manager.get_nonce_as_u256())
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
            chain_state.nonce_manager.unlock_nonce();
        }
    });
}

/// Releases the nonce lock and increments the nonce for a specific chain.
/// Optionally, the nonce can be explicitly set to `used_nonce` + 1 if provided.
pub fn release_and_increment_nonce(chain_id: u64, used_nonce: Option<u128>) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.nonce_manager.unlock_nonce();

            let new_nonce = used_nonce.unwrap_or(chain_state.nonce_manager.current_nonce) + 1;

            ic_cdk::println!(
                "[release_and_increment_nonce] prev nonce: {}, new nonce: {}",
                chain_state.nonce_manager.current_nonce,
                new_nonce
            );

            chain_state.nonce_manager.current_nonce = new_nonce;

            // Remove all unresolved nonces up to and including the current nonce
            chain_state
                .nonce_manager
                .unresolved_nonces
                .retain(|&nonce, _| nonce > new_nonce);
        }
    });
}

pub fn set_unresolved_nonce(chain_id: u64, used_nonce: Option<u128>, fees: NonceFeeEstimates) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            ic_cdk::println!(
                "[set_unresolved_nonce] chain_id: {}, used_nonce: {:?}",
                chain_id,
                used_nonce
            );

            let unresolved_nonce = used_nonce.unwrap_or(chain_state.nonce_manager.current_nonce);

            chain_state
                .nonce_manager
                .track_unresolved_nonce(unresolved_nonce, fees);
        }
    });
}

pub fn has_unresolved_nonces(chain_id: u64) -> bool {
    read_state(|state| {
        if let Some(chain_state) = state.chains.get(&chain_id) {
            if !chain_state.nonce_manager.unresolved_nonces.is_empty() {
                return true;
            }
        }

        false
    })
}

pub fn get_unresolved_nonce_fees(chain_id: u64, nonce: u128) -> Option<FeeEstimates> {
    read_state(|state| {
        if let Some(chain_state) = state.chains.get(&chain_id) {
            return chain_state
                .nonce_manager
                .get_unresolved_fee_estimates(nonce)
                .map(|fees| fees.into());
        }

        None
    })
}
