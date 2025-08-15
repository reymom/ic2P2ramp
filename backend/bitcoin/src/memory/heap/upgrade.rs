use std::{borrow::Cow, collections::HashMap};

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_btc_interface::Network;
use ic_stable_structures::{Storable, storable::Bound};

use crate::{
    memory::stable::HEAP_STATE,
    types::{RuneID, RuneMetadata},
};

use super::{
    config::{
        get_derivation_path, get_key_name, get_network, get_runes, set_derivation_path,
        set_key_name, set_network, set_runes,
    },
    init::UnisatConfig,
    state::{State, get_state, initialize_state},
};

const MAX_STATE_SIZE: u32 = 64 * 1024; // 64KB

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct UpdateArg {
    pub network: Option<Network>,
    pub btc_principal: Option<Principal>,
    pub unisat: Option<UnisatConfig>,
    pub proxy_url: Option<String>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct SerializableHeap {
    network: Network,
    derivation_path: Vec<Vec<u8>>,
    key_name: String,
    state: State,
    runes: HashMap<RuneID, RuneMetadata>,
}

impl Storable for SerializableHeap {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).expect("Failed to encode SerializableHeap"))
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).expect("Failed to encode SerializableHeap")
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("Failed to decode SerializableHeap")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_STATE_SIZE,
        is_fixed_size: false,
    };
}

impl SerializableHeap {
    pub fn from_internal(
        network: Network,
        derivation_path: Vec<Vec<u8>>,
        key_name: String,
        state: State,
        runes: HashMap<RuneID, RuneMetadata>,
    ) -> Self {
        SerializableHeap {
            network,
            derivation_path,
            key_name,
            state,
            runes,
        }
    }
}

/// Called before an upgrade to persist the current state.
pub fn pre_upgrade() {
    let serializable_state = SerializableHeap::from_internal(
        get_network(),
        get_derivation_path(),
        get_key_name(),
        get_state(),
        get_runes(),
    );

    HEAP_STATE.with(|heap| {
        heap.borrow_mut().insert(0, serializable_state);
    });
}

/// Called after an upgrade to restore the previous state and apply any updates.
pub fn post_upgrade(update_arg: Option<UpdateArg>) {
    HEAP_STATE.with_borrow(|heap| {
        if let Some(serializable_heap) = heap.get(&0) {
            set_network(serializable_heap.network);
            set_derivation_path(serializable_heap.derivation_path);
            set_key_name(serializable_heap.key_name);
            set_runes(serializable_heap.runes);

            let mut state = serializable_heap.state.clone();

            // Apply updates from UpdateArg, if any.
            if let Some(arg) = update_arg {
                update_state(arg, &mut state);
            }
            initialize_state(state);
        } else {
            ic_cdk::trap("Failed to restore state from upgrade");
        }
    });
}

/// Updates the state with values provided in the UpdateArg.
fn update_state(update_arg: UpdateArg, state: &mut State) {
    if let Some(network) = update_arg.network {
        set_network(network);
    }
    if let Some(btc_principal) = update_arg.btc_principal {
        state.btc_principal = btc_principal;
    }
    if let Some(proxy_url) = update_arg.proxy_url {
        state.proxy_url = proxy_url;
    }
    if let Some(unisat) = update_arg.unisat {
        state.unisat = unisat;
    }
}
