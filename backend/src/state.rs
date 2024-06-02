use crate::evm_rpc::{BlockTag, RpcService, RpcServices};

use candid::{CandidType, Deserialize};
use ethers_core::types::U256;
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Clone, Debug)]
pub struct State {
    pub rpc_services: RpcServices,
    pub rpc_service: RpcService,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub nonce: U256,
    pub block_tag: BlockTag,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidStateError {
    InvalidEthereumContractAddress(String),
    InvalidTopic(String),
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

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct InitArg {
    pub rpc_services: RpcServices,
    pub rpc_service: RpcService,
    pub ecdsa_key_id: EcdsaKeyId,
    pub block_tag: BlockTag,
    pub client_id: String,
    pub client_secret: String,
}

impl TryFrom<InitArg> for State {
    type Error = InvalidStateError;

    fn try_from(
        InitArg {
            rpc_services,
            rpc_service,
            ecdsa_key_id,
            block_tag,
            client_id,
            client_secret,
        }: InitArg,
    ) -> Result<Self, Self::Error> {
        let state = Self {
            rpc_services,
            rpc_service,
            ecdsa_pub_key: None,
            ecdsa_key_id,
            evm_address: None,
            nonce: U256::zero(),
            block_tag,
            client_id,
            client_secret,
        };
        Ok(state)
    }
}
