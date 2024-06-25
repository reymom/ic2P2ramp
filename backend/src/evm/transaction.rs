use std::time::Duration;

use super::rpc::{
    GetTransactionReceiptResult, MultiGetTransactionReceiptResult, MultiSendRawTransactionResult,
    RpcConfig, SendRawTransactionResult, SendRawTransactionStatus, CANISTER_ID,
};
use crate::state::read_state;

#[derive(Debug)]
pub enum TransactionStatus {
    Confirmed,
    Failed(String),
    Pending,
}

pub async fn send_raw_transaction(tx: String, chain_id: u64) -> SendRawTransactionStatus {
    let rpc_providers = read_state(|s| {
        s.rpc_services
            .get(&chain_id)
            .cloned()
            .ok_or("Unsupported chain ID")
    })
    .unwrap();
    let cycles = 10_000_000_000;

    let arg: Option<RpcConfig> = None;
    let res = ic_cdk::api::call::call_with_payment128(
        CANISTER_ID,
        "eth_sendRawTransaction",
        (rpc_providers, arg, tx),
        cycles,
    )
    .await;
    match res {
        Ok((res,)) => match res {
            MultiSendRawTransactionResult::Consistent(status) => match status {
                SendRawTransactionResult::Ok(status) => status,
                SendRawTransactionResult::Err(e) => {
                    ic_cdk::trap(format!("Error: {:?}", e).as_str());
                }
            },
            MultiSendRawTransactionResult::Inconsistent(_) => {
                ic_cdk::trap("Status is inconsistent");
            }
        },
        Err(e) => ic_cdk::trap(format!("Error: {:?}", e).as_str()),
    }
}

pub async fn check_transaction_status(tx_hash: String, chain_id: u64) -> TransactionStatus {
    let rpc_providers = read_state(|s| {
        s.rpc_services
            .get(&chain_id)
            .cloned()
            .ok_or("Unsupported chain ID")
    })
    .unwrap();

    let cycles = 10_000_000_000;
    let arg: Option<RpcConfig> = None;
    let res: Result<(MultiGetTransactionReceiptResult,), _> =
        ic_cdk::api::call::call_with_payment128(
            CANISTER_ID,
            "eth_getTransactionReceipt",
            (rpc_providers.clone(), arg, tx_hash.clone()),
            cycles,
        )
        .await;

    ic_cdk::println!("[check_transaction_receipt] res = {:?}", res);
    match res {
        Ok((MultiGetTransactionReceiptResult::Consistent(GetTransactionReceiptResult::Ok(
            Some(receipt),
        )),)) => {
            if receipt.status == 1_u32 {
                TransactionStatus::Confirmed
            } else {
                TransactionStatus::Failed(format!("Transaction failed: {:?}", receipt))
            }
        }
        Ok((MultiGetTransactionReceiptResult::Consistent(GetTransactionReceiptResult::Ok(
            None,
        )),)) => TransactionStatus::Pending,
        Ok(
            (MultiGetTransactionReceiptResult::Consistent(GetTransactionReceiptResult::Err(e)),),
        ) => TransactionStatus::Failed(format!("Error checking transaction: {:?}", e)),
        Ok((MultiGetTransactionReceiptResult::Inconsistent(_),)) => {
            TransactionStatus::Failed("Inconsistent status".to_string())
        }
        Err(e) => TransactionStatus::Failed(format!("Error checking transaction: {:?}", e)),
    }
}

pub fn spawn_transaction_checker<F>(
    tx_hash: String,
    chain_id: u64,
    max_attempts: u32,
    interval: Duration,
    on_success: F,
) where
    F: Fn() + 'static,
{
    fn schedule_check<F>(
        tx_hash: String,
        chain_id: u64,
        attempts: u32,
        max_attempts: u32,
        interval: Duration,
        on_success: F,
    ) where
        F: Fn() + 'static,
    {
        ic_cdk_timers::set_timer(interval, move || {
            ic_cdk::spawn(async move {
                match check_transaction_status(tx_hash.clone(), chain_id).await {
                    TransactionStatus::Confirmed => {
                        on_success();
                    }
                    TransactionStatus::Pending if attempts < max_attempts => {
                        ic_cdk::println!("[schedule_check] TransactionStatus::Pending...");
                        schedule_check(
                            tx_hash.clone(),
                            chain_id,
                            attempts + 1,
                            max_attempts,
                            interval,
                            on_success,
                        );
                    }
                    TransactionStatus::Failed(err) => {
                        ic_cdk::println!("[schedule_check] Transaction failed: {:?}", err);
                    }
                    _ => {
                        ic_cdk::println!(
                            "[schedule_check] Transaction status check exceeded maximum attempts"
                        );
                    }
                }
            });
        });
    }

    schedule_check(tx_hash, chain_id, 0, max_attempts, interval, on_success);
}
