use std::{borrow::Cow, collections::HashMap};

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::management_canister::ecdsa::EcdsaKeyId;
use ic_cdk_timers::TimerId;
use ic_stable_structures::{storable::Bound, Storable};

use crate::{
    management,
    model::{
        memory::stable::storage::HEAP_STATE,
        types::{
            evm::chains::ChainState,
            exchange_rate::ExchangeRateCache,
            payment::{paypal::PayPalState, revolut::RevolutState},
        },
    },
};

use super::{
    clear_order_timer, get_exchange_rate_cache, get_locked_order_timers, get_order_id_counter,
    get_state, get_user_id_counter,
    init::{ChainConfig, PaypalConfig, RevolutConfig},
    initialize_state, set_exchange_rate_cache, set_order_id_counter, set_order_timer,
    set_user_id_counter, State, LOCK_DURATION_TIME_SECONDS,
};

const MAX_HEAP_SIZE: u32 = 128 * 1024; // 128KB

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct UpdateArg {
    pub chains: Option<Vec<ChainConfig>>, // Optional chain configuration updates
    pub ecdsa_key_id: Option<EcdsaKeyId>, // Optional ECDSA key update
    pub paypal: Option<PaypalConfig>,     // Optional PayPal configuration update
    pub revolut: Option<RevolutConfig>,   // Optional Revolut configuration update
    pub proxy_url: Option<String>,        // Optional proxy URL update
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct SerializableHeap {
    user_id_counter: u64,
    order_id_counter: u64,
    locked_order_timers: HashMap<u64, u64>,
    exchange_rate_cache: HashMap<(String, String), ExchangeRateCache>,
    state: State,
}

impl Storable for SerializableHeap {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_HEAP_SIZE,
        is_fixed_size: false,
    };
}

impl SerializableHeap {
    pub fn from_internal(
        user_id_counter: u64,
        order_id_counter: u64,
        locked_order_timers: HashMap<u64, TimerId>,
        exchange_rate_cache: HashMap<(String, String), ExchangeRateCache>,
        state: State,
    ) -> Self {
        SerializableHeap {
            user_id_counter,
            order_id_counter,
            locked_order_timers: locked_order_timers
                .into_keys()
                .map(|order_id| {
                    (
                        order_id,
                        ic_cdk::api::time() + LOCK_DURATION_TIME_SECONDS * 1_000_000_000,
                    )
                })
                .collect(),
            exchange_rate_cache,
            state,
        }
    }

    pub fn set_locked_order_timers(self) {
        for (order_id, unlock_timestamp) in self.locked_order_timers {
            if ic_cdk::api::time() < unlock_timestamp {
                set_order_timer(order_id);
            } else {
                ic_cdk::spawn(async move {
                    if let Err(e) = management::order::unlock_order(order_id).await {
                        ic_cdk::println!("Failed to auto-unlock order {}: {:?}", order_id, e);
                    } else {
                        let _ = clear_order_timer(order_id);
                    }
                });
            }
        }
    }
}

pub fn pre_upgrade() {
    let serializable_state = SerializableHeap::from_internal(
        get_user_id_counter(),
        get_order_id_counter(),
        get_locked_order_timers(),
        get_exchange_rate_cache(),
        get_state(),
    );

    HEAP_STATE.with(|heap| {
        heap.borrow_mut().insert(0, serializable_state);
    });
}

pub fn post_upgrade(update_arg: Option<UpdateArg>) {
    HEAP_STATE.with_borrow(|heap| {
        if let Some(serializable_heap) = heap.get(&0) {
            set_user_id_counter(serializable_heap.user_id_counter);
            set_order_id_counter(serializable_heap.order_id_counter);
            serializable_heap.clone().set_locked_order_timers();
            set_exchange_rate_cache(serializable_heap.exchange_rate_cache);

            let mut state: State = serializable_heap.state.clone();
            if let Some(update_arg) = update_arg {
                update_state(update_arg, &mut state);
            }

            initialize_state(state);
        } else {
            ic_cdk::trap("Failed to restore heap state");
        }
    });
}

fn update_state(update_arg: UpdateArg, state: &mut State) {
    // Update or add chains
    if let Some(chains) = update_arg.chains {
        for config in chains {
            state
                .chains
                .entry(config.chain_id)
                .and_modify(|chain_state| {
                    chain_state
                        .vault_manager_address
                        .clone_from(&config.vault_manager_address);
                    chain_state.rpc_services = config.services.clone();
                })
                .or_insert(ChainState::new(
                    config.vault_manager_address,
                    config.services,
                    config.currency_symbol,
                ));
        }
    }

    if let Some(ecdsa_key_id) = update_arg.ecdsa_key_id {
        state.ecdsa_key_id = ecdsa_key_id;
    };

    // Update PayPal
    if let Some(paypal_config) = update_arg.paypal {
        state.paypal = PayPalState {
            access_token: None, // Reset access token on upgrade
            token_expiration: None,
            client_id: paypal_config.client_id,
            client_secret: paypal_config.client_secret,
            api_url: paypal_config.api_url,
        };
    }

    if let Some(revolut_config) = update_arg.revolut {
        state.revolut = RevolutState {
            access_token: None, // Reset access token on update
            token_expiration: None,
            client_id: revolut_config.client_id,
            api_url: revolut_config.api_url,
            proxy_url: revolut_config.proxy_url,
            private_key_der: revolut_config.private_key_der,
            kid: revolut_config.kid,
            tan: revolut_config.tan,
        };
    }

    if let Some(proxy_url) = update_arg.proxy_url {
        state.proxy_url = proxy_url;
    }
}
