use ic_cdk_timers::TimerId;
use std::{cell::RefCell, collections::HashMap};

use super::State;
use crate::model::types::evm::logs::EvmTransactionLog;

pub mod logs;

const LOCK_DURATION_TIME_SECONDS: u64 = 120; // 1 hour

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();

    static USER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
    static ORDER_ID_COUNTER: RefCell<u64> = RefCell::new(0);
    static LOCKED_ORDER_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::default();

    static EVM_TRANSACTION_LOGS: RefCell<HashMap<u64, EvmTransactionLog>> = RefCell::new(HashMap::new());
    static TRANSACTION_LOG_TIMERS: RefCell<HashMap<u64, TimerId>> = RefCell::new(HashMap::new());
}
