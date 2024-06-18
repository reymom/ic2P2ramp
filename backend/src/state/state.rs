use candid::{CandidType, Deserialize};
use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::{cell::RefCell, collections::HashMap};

use crate::evm::rpc::RpcServices;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Clone, Debug)]
pub struct State {
    pub vault_manager_addresses: HashMap<u64, String>,
    pub rpc_services: HashMap<u64, RpcServices>,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub nonce: U256,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidStateError {
    // InvalidEthereumContractAddress(String),
    // InvalidTopic(String),
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

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct RpcServiceConfig {
    pub chain_id: u64,
    pub services: RpcServices,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub rpc_services: Vec<RpcServiceConfig>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub client_id: String,
    pub client_secret: String,
    pub vault_manager_addresses: Vec<(u64, String)>,
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;

    fn try_from(
        InitArg {
            rpc_services,
            ecdsa_key_id,
            client_id,
            client_secret,
            vault_manager_addresses,
        }: InitArg,
    ) -> Result<Self, Self::Error> {
        let mut rpc_services_map = HashMap::new();
        for config in rpc_services {
            rpc_services_map.insert(config.chain_id, config.services);
        }

        let mut vault_manager_map = HashMap::new();
        for (chain_id, address) in vault_manager_addresses {
            vault_manager_map.insert(chain_id, address);
        }

        let state = Self {
            vault_manager_addresses: vault_manager_map,
            rpc_services: rpc_services_map,
            ecdsa_pub_key: None,
            ecdsa_key_id,
            evm_address: None,
            nonce: U256::zero(),
            client_id,
            client_secret,
        };
        Ok(state)
    }
}
