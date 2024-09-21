use std::time::Duration;

use candid::CandidType;
use ethers_core::types::U256;

use super::{
    fees::FeeEstimates,
    rpc::{
        BlockTag, GetTransactionCountArgs, GetTransactionCountResult, GetTransactionReceiptResult,
        MultiGetTransactionCountResult, MultiGetTransactionReceiptResult,
        MultiSendRawTransactionResult, RpcConfig, SendRawTransactionResult,
        SendRawTransactionStatus, TransactionReceipt, EVM_RPC,
    },
    signer::{self, SignRequest},
};

use crate::{
    errors::{RampError, Result},
    model::{
        helpers,
        memory::heap::{logs, read_state},
        types::evm::chains,
    },
    types::evm::{
        chains::{get_rpc_providers, increment_nonce},
        logs::TransactionAction,
    },
};

#[derive(Debug, Clone, CandidType)]
pub enum TransactionStatus {
    Confirmed(TransactionReceipt),
    Failed(String),
    Pending,
}

pub(crate) const MAX_RETRY_ATTEMPTS: u8 = 4;
pub(crate) const MAX_ATTEMPTS_PER_RETRY: u16 = 40;
pub(crate) const ATTEMPT_INTERVAL_SECONDS: u64 = 4;

pub async fn create_sign_request(
    abi: &str,
    function_name: &str,
    gas: U256,
    fee_estimates: FeeEstimates,
    chain_id: u64,
    value: U256,
    to_address: String,
    inputs: &[ethers_core::abi::Token],
) -> Result<SignRequest> {
    let contract = ethers_core::abi::Contract::load(abi.as_bytes())
        .map_err(|e| RampError::EthersAbiError(format!("Contract load error: {:?}", e)))?;
    let function = contract
        .function(function_name)
        .map_err(|e| RampError::EthersAbiError(format!("Function not found error: {:?}", e)))?;
    let data = function
        .encode_input(inputs)
        .map_err(|e| RampError::EthersAbiError(format!("Encode input error: {:?}", e)))?;

    Ok(signer::create_sign_request(
        value,
        chain_id.into(),
        Some(to_address.clone()),
        None,
        gas,
        Some(data),
        fee_estimates,
    )
    .await)
}

pub async fn send_signed_transaction(request: SignRequest, chain_id: u64) -> Result<String> {
    let tx = signer::sign_transaction(request).await;

    match send_raw_transaction(tx.clone(), chain_id).await {
        SendRawTransactionStatus::Ok(transaction_hash) => {
            ic_cdk::println!("[send_signed_transactions] tx_hash = {transaction_hash:?}");
            increment_nonce(chain_id);
            transaction_hash.ok_or(RampError::EmptyTransactionHash)
        }
        SendRawTransactionStatus::NonceTooLow => Err(RampError::NonceTooLow),
        SendRawTransactionStatus::NonceTooHigh => Err(RampError::NonceTooHigh),
        SendRawTransactionStatus::InsufficientFunds => Err(RampError::InsufficientFunds),
    }
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
    let cycles = 10_000_000_000;
    let rpc_providers = get_rpc_providers(chain_id);
    let arg: Option<RpcConfig> = None;
    let res = EVM_RPC
        .eth_get_transaction_receipt(rpc_providers, arg, tx_hash, cycles)
        .await;

    ic_cdk::println!("[check_transaction_receipt] res = {:?}", res);
    match res {
        Ok((MultiGetTransactionReceiptResult::Consistent(GetTransactionReceiptResult::Ok(
            Some(receipt),
        )),)) => {
            if receipt.status == 1_u32 {
                TransactionStatus::Confirmed(receipt)
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

pub async fn _wait_for_transaction_confirmation(
    tx_hash: String,
    chain_id: u64,
    max_attempts: u32,
    interval: Duration,
) -> Result<()> {
    for attempt in 0..max_attempts {
        match check_transaction_status(tx_hash.clone(), chain_id).await {
            TransactionStatus::Confirmed(_) => {
                return Ok(());
            }
            TransactionStatus::Failed(err) => {
                return Err(RampError::_TransactionFailed(err));
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
        helpers::delay(interval).await;
    }
    Err(RampError::TransactionTimeout)
}

pub fn spawn_transaction_checker<F>(
    retry_attempt: u8,
    tx_hash: String,
    chain_id: u64,
    order_id: u64,
    action: Option<TransactionAction>,
    sign_request: SignRequest,
    on_success: F,
) where
    F: Fn(TransactionReceipt) + 'static,
{
    fn schedule_check<F>(
        tx_hash: String,
        chain_id: u64,
        attempt: u16,
        retry_attempt: u8,
        order_id: u64,
        sign_request: SignRequest,
        on_success: F,
    ) where
        F: Fn(TransactionReceipt) + 'static,
    {
        ic_cdk_timers::set_timer(Duration::from_secs(ATTEMPT_INTERVAL_SECONDS), move || {
            ic_cdk::println!("[schedule_check] spawning...");
            ic_cdk::spawn(async move {
                match check_transaction_status(tx_hash.clone(), chain_id).await {
                    TransactionStatus::Confirmed(receipt) => {
                        ic_cdk::println!("[schedule_check] TransactionStatus::Confirmed");
                        logs::update_transaction_log(
                            order_id,
                            TransactionStatus::Confirmed(receipt.clone()),
                        );
                        on_success(receipt);
                    }
                    TransactionStatus::Pending if attempt < MAX_ATTEMPTS_PER_RETRY - 1 => {
                        ic_cdk::println!("[schedule_check] TransactionStatus::Pending...");
                        logs::update_transaction_log(order_id, TransactionStatus::Pending);

                        schedule_check(
                            tx_hash.clone(),
                            chain_id,
                            attempt + 1,
                            retry_attempt,
                            order_id,
                            sign_request,
                            on_success,
                        );
                    }
                    TransactionStatus::Pending if attempt == MAX_ATTEMPTS_PER_RETRY - 1 => {
                        ic_cdk::spawn(async move {
                            retry_with_bumped_fees(
                                sign_request,
                                chain_id,
                                order_id,
                                retry_attempt,
                                on_success,
                            )
                            .await;
                        });
                    }
                    TransactionStatus::Failed(err) => {
                        ic_cdk::println!("[schedule_check] Transaction failed: {:?}", err);
                        logs::update_transaction_log(order_id, TransactionStatus::Failed(err));
                    }
                    _ => {
                        ic_cdk::println!(
                            "[schedule_check] Transaction status check exceeded maximum attempts"
                        );
                        logs::remove_transaction_log(order_id);
                    }
                }
            });
        });
    }

    if action.is_none() && retry_attempt == 0 {
        ic_cdk::println!("action is not defined, schedule check stopped");
        return;
    } else if retry_attempt == 0 {
        logs::add_transaction_log(order_id, action.unwrap());
    }

    if retry_attempt < MAX_RETRY_ATTEMPTS {
        schedule_check(
            tx_hash,
            chain_id,
            0,
            retry_attempt,
            order_id,
            sign_request,
            on_success,
        );
    }
}

pub async fn retry_with_bumped_fees<F>(
    mut sign_request: SignRequest,
    chain_id: u64,
    order_id: u64,
    retry_attempt: u8,
    on_success: F,
) where
    F: Fn(TransactionReceipt) + 'static,
{
    ic_cdk::println!("[retry_with_bumped_fees] Retrying transaction with bumped fees.");

    // Bump gas fees
    sign_request.max_fee_per_gas = Some(bump_fee(sign_request.max_fee_per_gas));
    sign_request.max_priority_fee_per_gas = Some(bump_fee(sign_request.max_priority_fee_per_gas));

    // Resend the transaction with updated fees
    match send_signed_transaction(sign_request.clone(), chain_id).await {
        Ok(new_tx_hash) => {
            ic_cdk::println!(
                "[retry_with_bumped_fees] New transaction hash: {}",
                new_tx_hash
            );
            // Spawn a new checker for the retried transaction
            spawn_transaction_checker(
                retry_attempt + 1,
                new_tx_hash,
                chain_id,
                order_id,
                None,
                sign_request,
                on_success,
            );
        }
        Err(e) => {
            ic_cdk::println!(
                "[retry_with_bumped_fees] Failed to send transaction: {:?}",
                e
            );
        }
    }
}

fn bump_fee(current_fee: Option<U256>) -> U256 {
    // Bump gas fee 20%
    current_fee.unwrap_or(U256::zero()) * U256::from(12) / U256::from(10)
}

pub async fn eth_get_transaction_count(chain_id: u64) -> Result<u128> {
    let rpc_providers = chains::get_rpc_providers(chain_id);
    let address = read_state(|s| s.evm_address.clone()).expect("evm address should be initialized");

    let cycles = 10_000_000_000;
    match EVM_RPC
        .eth_get_transaction_count(
            rpc_providers,
            None,
            GetTransactionCountArgs {
                address,
                block: BlockTag::Latest,
            },
            cycles,
        )
        .await
    {
        Ok((res,)) => match res {
            MultiGetTransactionCountResult::Consistent(block_result) => match block_result {
                GetTransactionCountResult::Ok(count) => Ok(count),
                GetTransactionCountResult::Err(e) => Err(RampError::RpcError(format!("{:?}", e))),
            },
            MultiGetTransactionCountResult::Inconsistent(_) => {
                ic_cdk::trap("Block Result is inconsistent");
            }
        },
        Err((code, message)) => Err(RampError::ICRejectionError(code, message)),
    }
}
