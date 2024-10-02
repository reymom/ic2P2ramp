use std::time::Duration;

use ic_cdk_timers::{clear_timer, set_timer};

use crate::model::types::evm::{
    logs::{EvmTransactionLog, TransactionStatus},
    transaction::TransactionAction,
};

use super::storage::{EVM_TRANSACTION_LOGS, TRANSACTION_LOG_TIMERS};

pub fn new_transaction_log(order_id: u64, action: TransactionAction) {
    EVM_TRANSACTION_LOGS.with_borrow_mut(|logs| {
        logs.insert(
            order_id,
            EvmTransactionLog {
                order_id,
                action,
                status: TransactionStatus::Broadcasting,
            },
        );
    });
}

pub fn update_transaction_log(order_id: u64, status: TransactionStatus) {
    ic_cdk::println!(
        "[update_transaction_log] order_id: {}, new status: {:?}",
        order_id,
        status
    );
    EVM_TRANSACTION_LOGS.with_borrow_mut(|logs| {
        if let Some(log) = logs.get_mut(&order_id) {
            log.status = status.clone();
        }
    });

    if matches!(status, TransactionStatus::Confirmed(_)) {
        set_transaction_removal_timer(order_id);
    }
}

pub fn remove_transaction_log(order_id: u64) {
    EVM_TRANSACTION_LOGS.with_borrow_mut(|logs| {
        logs.remove(&order_id);
    });
}

pub fn get_transaction_log(order_id: u64) -> Option<EvmTransactionLog> {
    EVM_TRANSACTION_LOGS.with_borrow(|logs| logs.get(&order_id).cloned())
}

pub fn get_pending_transactions() -> Vec<EvmTransactionLog> {
    EVM_TRANSACTION_LOGS.with_borrow(|logs| {
        logs.iter()
            .filter_map(|(_, log)| match &log.status {
                TransactionStatus::Pending => Some(log.clone()),
                _ => None,
            })
            .collect()
    })
}

pub fn set_transaction_removal_timer(order_id: u64) {
    let timer_id = set_timer(Duration::from_secs(300), move || {
        remove_transaction_log(order_id);
        clear_transaction_timer(order_id);
    });

    TRANSACTION_LOG_TIMERS.with_borrow_mut(|timers| {
        timers.insert(order_id, timer_id);
    });
}

pub fn clear_transaction_timer(order_id: u64) {
    TRANSACTION_LOG_TIMERS.with_borrow_mut(|timers| {
        if let Some(timer_id) = timers.remove(&order_id) {
            clear_timer(timer_id);
        }
    });
}
