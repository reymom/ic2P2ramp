use std::collections::HashMap;

use candid::{CandidType, Deserialize};
use ethers_core::types::U256;

use crate::{
    errors::{BlockchainError, Result, SystemError},
    evm::rpc::RpcServices,
    model::{
        helpers,
        memory::heap::{mutate_state, read_state},
    },
};

use super::{gas::ChainGasTracking, token::Token};

pub const LOCK_NONCE_TIME_SECONDS: u64 = 5;

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

    pub fn lock_nonce(&mut self) {
        self.nonce_locked = true;
    }

    pub fn unlock_nonce(&mut self) {
        self.nonce_locked = false;
    }
}

pub fn get_nonce(chain_id: u64) -> Result<U256> {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.lock_nonce();
            Ok(chain_state.get_nonce_as_u256())
        } else {
            Err(BlockchainError::ChainIdNotFound(chain_id).into())
        }
    })
}

pub fn release_nonce(chain_id: u64) {
    mutate_state(|state| {
        if let Some(chain_state) = state.chains.get_mut(&chain_id) {
            chain_state.unlock_nonce();
            chain_state.increment_nonce();
        }
    });
}

pub fn get_rpc_providers(chain_id: u64) -> Result<RpcServices> {
    read_state(|s| {
        s.chains
            .get(&chain_id)
            .map(|chain_state| chain_state.rpc_services.clone())
            .ok_or_else(|| BlockchainError::ChainIdNotFound(chain_id).into())
    })
}

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

pub fn chain_is_supported(chain_id: u64) -> Result<()> {
    read_state(|state| {
        if state.chains.contains_key(&chain_id) {
            Ok(())
        } else {
            Err(BlockchainError::ChainIdNotFound(chain_id).into())
        }
    })
}

pub async fn wait_for_nonce_unlock(chain_id: u64) -> Result<()> {
    let start_time = ic_cdk::api::time();

    loop {
        let is_locked = read_state(|s| {
            s.chains
                .get(&chain_id)
                .map(|chain_state| chain_state.nonce_locked)
                .unwrap_or(false)
        });

        if !is_locked {
            return Ok(());
        }

        if ic_cdk::api::time() - start_time > LOCK_NONCE_TIME_SECONDS * 1_000_000_000 {
            return Err(BlockchainError::NonceLockTimeout(chain_id).into());
        }

        ic_cdk::println!("[wait_for_nonce_unlock] Nonce is locked, waiting...");
        helpers::delay(std::time::Duration::from_millis(60)).await;
    }
}
