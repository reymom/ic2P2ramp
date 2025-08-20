use std::{borrow::Cow, collections::HashMap};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use icramp_types::solana::token::TokenInfo;
use sol_rpc_types::SolanaCluster;

use crate::memory::{
    heap::{
        config::{get_state, get_tokens, init_state, set_tokens},
        state::State,
    },
    stable::HEAP_STATE,
};

const MAX_STATE_SIZE: u32 = 64 * 1024; // 64KB

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct UpdateArg {
    pub network: Option<SolanaCluster>,
    pub sol_rpc_canister_id: Option<Principal>,
    pub proxy_url: Option<String>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SerializableHeap {
    state: State,
    tokens: HashMap<String, TokenInfo>,
}

impl Storable for SerializableHeap {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).expect("Failed to encode SerializableState"))
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).expect("Failed to encode SerializableHeap")
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode SerializableState")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_STATE_SIZE,
        is_fixed_size: false,
    };
}

impl SerializableHeap {
    pub fn from_internal(state: State, tokens: HashMap<String, TokenInfo>) -> Self {
        SerializableHeap { state, tokens }
    }
}

/// Called before an upgrade to persist the current state.
pub fn pre_upgrade() {
    let serializable_state = SerializableHeap::from_internal(get_state(), get_tokens());

    HEAP_STATE.with(|heap| {
        heap.borrow_mut().insert(0, serializable_state);
    });
}

/// Called after an upgrade to restore the previous state and apply any updates.
pub fn post_upgrade(update_arg: Option<UpdateArg>) {
    HEAP_STATE.with_borrow(|heap| {
        if let Some(serializable_heap) = heap.get(&0) {
            set_tokens(serializable_heap.tokens);

            // Apply updates from UpdateArg, if any.
            let mut state = serializable_heap.state.clone();
            if let Some(arg) = update_arg {
                update_state(arg, &mut state);
            }
            init_state(state);
        } else {
            ic_cdk::trap("Failed to restore state from upgrade");
        }
    });
}

/// Updates the state with values provided in the UpdateArg.
fn update_state(update_arg: UpdateArg, state: &mut State) {
    if let Some(sol_rpc_canister_id) = update_arg.sol_rpc_canister_id {
        state.sol_rpc_canister_id = sol_rpc_canister_id;
    }
    if let Some(proxy_url) = update_arg.proxy_url {
        state.proxy_url = proxy_url;
    }
}
