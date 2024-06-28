use std::time::Duration;

use super::rpc::{
    GetTransactionReceiptResult, MultiGetTransactionReceiptResult, MultiSendRawTransactionResult,
    RpcConfig, SendRawTransactionResult, SendRawTransactionStatus, EVM_RPC,
};
use crate::{
    errors::{RampError, Result},
    state::chains::get_rpc_providers,
};

#[derive(Debug)]
pub enum TransactionStatus {
    Confirmed,
    Failed(String),
    Pending,
}

pub async fn send_raw_transaction(tx: String, chain_id: u64) -> SendRawTransactionStatus {
    let rpc_providers = get_rpc_providers(chain_id);
    let cycles = 10_000_000_000;

    let arg: Option<RpcConfig> = Some(RpcConfig {
        responseSizeEstimate: Some(1024),
    });
    match EVM_RPC
        .send_raw_transaction(rpc_providers, arg, tx, cycles)
        .await
    {
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
    let rpc_providers = get_rpc_providers(chain_id);
    let arg: Option<RpcConfig> = None;
    let res = EVM_RPC
        .eth_get_transaction_receipt(rpc_providers, arg, tx_hash)
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

pub async fn wait_for_transaction_confirmation(
    tx_hash: String,
    chain_id: u64,
    max_attempts: u32,
    interval: Duration,
) -> Result<()> {
    for attempt in 0..max_attempts {
        match check_transaction_status(tx_hash.clone(), chain_id).await {
            TransactionStatus::Confirmed => {
                return Ok(());
            }
            TransactionStatus::Failed(err) => {
                return Err(RampError::TransactionFailed(err));
            }
            TransactionStatus::Pending => {
                if attempt + 1 >= max_attempts {
                    return Err(RampError::TransactionTimeout);
                }
                ic_cdk::println!(
                    "[wait_for_transaction_confirmation] Transaction is pending in attempt={:?}",
                    attempt
                );
            }
        }
        super::helpers::delay(interval).await;
    }
    Err(RampError::TransactionTimeout)
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
