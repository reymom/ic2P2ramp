use candid::{CandidType, Deserialize};
use ic_cdk_timers::{set_timer, TimerId};
use std::{collections::HashMap, time::Duration};

use crate::management;

use super::{
    get_locked_order_timers, get_order_id_counter, get_user_id_counter, set_locked_order_timers,
    set_order_id_counter, set_user_id_counter,
};

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SerializableState {
    pub user_id_counter: u64,
    pub order_id_counter: u64,
    pub locked_order_timers: HashMap<u64, u64>,
}

impl SerializableState {
    pub fn from_internal(
        user_id_counter: u64,
        order_id_counter: u64,
        locked_order_timers: HashMap<u64, TimerId>,
    ) -> Self {
        SerializableState {
            user_id_counter,
            order_id_counter,
            locked_order_timers: locked_order_timers
                .into_iter()
                .map(|(order_id, _)| (order_id, ic_cdk::api::time() + 3600))
                .collect(),
        }
    }

    pub fn into_internal(self) -> HashMap<u64, TimerId> {
        self.locked_order_timers
            .into_iter()
            .filter_map(|(order_id, unlock_timestamp)| {
                if ic_cdk::api::time() < unlock_timestamp {
                    let remaining_duration = unlock_timestamp - ic_cdk::api::time();
                    let timer_id = set_timer(Duration::from_secs(remaining_duration), move || {
                        ic_cdk::spawn(async move {
                            if let Err(e) = management::order::unlock_order(order_id, None).await {
                                ic_cdk::println!(
                                    "Failed to auto-unlock order {}: {:?}",
                                    order_id,
                                    e
                                );
                            }
                        });
                    });
                    Some((order_id, timer_id))
                } else {
                    // If the timer has already expired, unlock the order immediately
                    ic_cdk::spawn(async move {
                        if let Err(e) = management::order::unlock_order(order_id, None).await {
                            ic_cdk::println!("Failed to auto-unlock order {}: {:?}", order_id, e);
                        }
                    });
                    None
                }
            })
            .collect()
    }
}

pub fn pre_upgrade() {
    let serializable_state = SerializableState::from_internal(
        get_user_id_counter(),
        get_order_id_counter(),
        get_locked_order_timers(),
    );

    ic_cdk::storage::stable_save((serializable_state,)).expect("failed to save state");
}

pub fn post_upgrade() {
    // Restore the SerializableState directly from stable memory
    let (serializable_state,): (SerializableState,) =
        ic_cdk::storage::stable_restore().expect("failed to restore state");

    set_user_id_counter(serializable_state.user_id_counter);
    set_order_id_counter(serializable_state.order_id_counter);
    set_locked_order_timers(serializable_state.into_internal());
}
