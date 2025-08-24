use candid::{CandidType, Deserialize, Principal};
use icramp_types::bitcoin::setup::UnisatConfig;

use super::config::STATE;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct State {
    pub btc_principal: Principal,
    pub proxy_url: String,
    pub unisat: UnisatConfig,
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
