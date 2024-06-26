use candid::{CandidType, Deserialize};
use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::{cell::RefCell, collections::HashMap, str::FromStr};

use crate::evm::rpc::RpcServices;

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

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub vault_manager_address: String,
    pub services: RpcServices,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub chains: Vec<ChainConfig>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub client_id: String,
    pub client_secret: String,
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;

    fn try_from(
        InitArg {
            chains,
            ecdsa_key_id,
            client_id,
            client_secret,
        }: InitArg,
    ) -> Result<Self, Self::Error> {
        let mut chains_map = HashMap::new();
        for config in chains {
            ethers_core::types::Address::from_str(&config.vault_manager_address).map_err(|e| {
                InvalidStateError::InvalidEthereumContractAddress(format!("ERROR: {}", e))
            })?;

            chains_map.insert(
                config.chain_id,
                ChainState {
                    vault_manager_address: config.vault_manager_address,
                    rpc_services: config.services,
                    nonce: U256::zero(),
                    approved_tokens: HashMap::new(),
                },
            );
        }

        let state = Self {
            chains: chains_map,
            ecdsa_pub_key: None,
            ecdsa_key_id,
            evm_address: None,
            client_id,
            client_secret,
        };
        Ok(state)
    }
}
