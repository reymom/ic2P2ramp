use std::collections::HashMap;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;

use crate::model::types::{
    evm::chains::ChainState,
    icp::IcpToken,
    ordiscan::OrdiscanState,
    payment::{paypal::PayPalState, revolut::RevolutState, stripe::StripeState},
    unisat::UnisatState,
};

use super::storage::STATE;

/// Canister ids to make intercall canisters
#[derive(Clone, CandidType, Deserialize)]
pub struct CanisterIds {
    pub solana_backend_id: Principal,
    pub bitcoin_backend_id: Principal,
}

#[derive(Clone, CandidType, Deserialize)]
pub struct State {
    pub canister_ids: CanisterIds,
    pub chains: HashMap<u64, ChainState>,
    pub ecdsa_pub_key: Option<Vec<u8>>,
    pub ecdsa_key_id: EcdsaKeyId,
    pub evm_address: Option<String>,
    pub paypal: PayPalState,
    pub stripe: StripeState,
    pub revolut: RevolutState,
    pub proxy_url: String,
    pub ordiscan: OrdiscanState,
    pub unisat: UnisatState,
    pub icp_tokens: HashMap<Principal, IcpToken>,
}

impl std::fmt::Debug for CanisterIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CanisterIds")
            .field("solana_backend_id", &self.solana_backend_id.to_text())
            .field("bitcoin_backend_id", &self.bitcoin_backend_id.to_text())
            .finish()
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("canister_ids", &self.canister_ids) // CanisterIds gets custom Debug too
            .field("chains", &self.chains)
            .field("ecdsa_pub_key", &self.ecdsa_pub_key)
            .field("ecdsa_key_id", &self.ecdsa_key_id)
            .field("evm_address", &self.evm_address)
            .field("paypal", &self.paypal)
            .field("stripe", &self.stripe)
            .field("revolut", &self.revolut)
            .field("proxy_url", &self.proxy_url)
            .field("ordiscan", &self.ordiscan)
            .field("unisat", &self.unisat)
            // For icp_tokens, print principals as text
            .field(
                "icp_tokens",
                &self
                    .icp_tokens
                    .iter()
                    .map(|(p, t)| (p.to_text(), t))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum InvalidStateError {
    InvalidEthereumContractAddress(String),
    InvalidCanisterId(String),
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

pub fn get_state() -> State {
    STATE.with_borrow(|state| {
        state
            .as_ref()
            .expect("BUG: state is not initialized")
            .clone()
    })
}
