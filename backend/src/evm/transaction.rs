use std::time::Duration;

use ethers_core::{abi, types::U256};

use super::{
    rpc::{
        BlockTag, GetTransactionCountArgs, GetTransactionCountResult, GetTransactionReceiptResult,
        MultiGetTransactionCountResult, MultiGetTransactionReceiptResult,
        MultiSendRawTransactionResult, RpcConfig, RpcError, SendRawTransactionResult,
        SendRawTransactionStatus, TransactionReceipt, EVM_RPC,
    },
    signer,
};
use crate::{
    errors::{BlockchainError, RampError, Result, SystemError},
    evm::{fees, helper::load_contract_data, vault::Ic2P2ramp},
    management::vault,
    model::memory::{
        heap::{logs, read_state},
        stable::orders::unset_processing_order,
    },
    types::{
        evm::{
            chains::{self, get_rpc_providers, release_and_increment_nonce, release_nonce},
            logs::TransactionStatus,
            request::SignRequest,
            transaction::TransactionAction,
        },
        orders::LockInput,
    },
};

pub(crate) const MAX_RETRY_ATTEMPTS: u8 = 4;
pub(crate) const MAX_ATTEMPTS_PER_RETRY: u16 = 40;
pub(crate) const ATTEMPT_INTERVAL_SECONDS: u64 = 4;

pub fn broadcast_transaction(
    order_id: u64,
    chain_id: u64,
    action: TransactionAction,
    sign_request: SignRequest,
    lock_input: Option<LockInput>,
    attempt: u8,
    nonce_retry: bool,
) {
    ic_cdk_timers::set_timer(Duration::from_millis(200), move || {
        ic_cdk::spawn(async move {
            ic_cdk::println!("[broadcast_transaction] attempt = {}", attempt);
            if nonce_retry || !chains::nonce_locked(chain_id) {
                let mut sign_request = sign_request;
                if !nonce_retry {
                    let nonce = chains::get_and_lock_nonce(chain_id).unwrap();
                    sign_request.add_nonce(nonce);
                }

                match send_signed_transaction(sign_request.clone(), chain_id).await {
                    Ok(tx_hash) => {
                        release_and_increment_nonce(
                            chain_id,
                            sign_request.nonce.map(|nonce| nonce.as_u128()),
                        );
                        logs::update_transaction_log(
                            order_id,
                            TransactionStatus::Broadcasted(
                                tx_hash.clone(),
                                sign_request.clone().into(),
                            ),
                        );

                        match action {
                            TransactionAction::Commit => {
                                let lock_input = match lock_input {
                                    Some(input) => input,
                                    None => {
                                        release_nonce(chain_id);
                                        let _ = unset_processing_order(&order_id);
                                        logs::update_transaction_log(
                                            order_id,
                                            TransactionStatus::BroadcastError(
                                                SystemError::InvalidInput(
                                                    "Lock Input not found".to_string(),
                                                )
                                                .into(),
                                            ),
                                        );
                                        return;
                                    }
                                };
                                vault::spawn_commit_listener(
                                    order_id,
                                    chain_id,
                                    &tx_hash,
                                    sign_request,
                                    lock_input,
                                );
                            }
                            TransactionAction::Uncommit => vault::spawn_uncommit_listener(
                                order_id,
                                chain_id,
                                &tx_hash,
                                sign_request,
                            ),
                            TransactionAction::Cancel(cancel_variant) => {
                                if order_id != 0 {
                                    vault::spawn_cancel_listener(
                                        order_id,
                                        chain_id,
                                        cancel_variant,
                                        &tx_hash,
                                        sign_request,
                                    )
                                } else {
                                    ic_cdk::println!("Broadcasted tx: {}", tx_hash);
                                    logs::update_transaction_log(
                                        order_id,
                                        TransactionStatus::Confirmed(TransactionReceipt::default()),
                                    );
                                }
                            }
                            TransactionAction::Release(release_variant) => {
                                vault::spawn_release_listener(
                                    order_id,
                                    chain_id,
                                    release_variant,
                                    &tx_hash,
                                    sign_request,
                                )
                            }
                            TransactionAction::Transfer(..) => {
                                ic_cdk::println!("Broadcasted tx: {}", tx_hash);
                                logs::update_transaction_log(
                                    order_id,
                                    TransactionStatus::Confirmed(TransactionReceipt::default()),
                                );
                            }
                        }
                    }

                    Err(RampError::BlockchainError(BlockchainError::NonceTooLow)) => {
                        match eth_get_transaction_count(chain_id).await {
                            Ok(tx_count) => {
                                let sign_request = SignRequest {
                                    nonce: Some(tx_count.into()),
                                    ..sign_request
                                };
                                broadcast_transaction(
                                    order_id,
                                    chain_id,
                                    action,
                                    sign_request,
                                    lock_input,
                                    attempt + 1,
                                    true,
                                );
                            }
                            Err(e) => {
                                release_nonce(chain_id);
                                let _ = unset_processing_order(&order_id);
                                logs::update_transaction_log(
                                    order_id,
                                    TransactionStatus::BroadcastError(e),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        release_nonce(chain_id);
                        let _ = unset_processing_order(&order_id);
                        logs::update_transaction_log(
                            order_id,
                            TransactionStatus::BroadcastError(e),
                        );
                    }
                }
            } else if attempt > 20 {
                release_nonce(chain_id);
                let _ = unset_processing_order(&order_id);
                logs::update_transaction_log(
                    order_id,
                    TransactionStatus::BroadcastError(
                        BlockchainError::NonceLockTimeout(chain_id).into(),
                    ),
                )
            } else {
                broadcast_transaction(
                    order_id,
                    chain_id,
                    action,
                    sign_request,
                    lock_input,
                    attempt + 1,
                    false,
                );
            }
        })
    });
}

pub(super) async fn create_vault_sign_request(
    chain_id: u64,
    transaction_type: &TransactionAction,
    inputs: &[abi::Token],
    estimated_gas: Option<u64>,
) -> Result<SignRequest> {
    let gas = U256::from(Ic2P2ramp::get_final_gas(
        estimated_gas.unwrap_or(transaction_type.default_gas(chain_id)),
    ));

    let fee_estimates = fees::get_fee_estimates(9, chain_id).await?;
    ic_cdk::println!(
        "[create_sign_request_without_nonce] gas = {:?}, max_fee_per_gas = {:?}, max_priority_fee_per_gas = {:?}",
        gas,
        fee_estimates.max_fee_per_gas,
        fee_estimates.max_priority_fee_per_gas
    );

    let data = load_contract_data(
        transaction_type.abi(),
        transaction_type.function_name(),
        inputs,
    )?;

    let vault_manager_address = chains::get_vault_manager_address(chain_id)?;
    Ok(SignRequest {
        chain_id: Some(chain_id.into()),
        to: Some(vault_manager_address),
        from: None,
        gas,
        max_fee_per_gas: Some(fee_estimates.max_fee_per_gas),
        max_priority_fee_per_gas: Some(fee_estimates.max_priority_fee_per_gas),
        data: Some(data),
        value: None,
        nonce: None,
    })
}

async fn send_signed_transaction(request: SignRequest, chain_id: u64) -> Result<String> {
    let tx = signer::sign_transaction(request).await;

    match send_raw_transaction(tx.clone(), chain_id).await? {
        SendRawTransactionStatus::Ok(transaction_hash) => {
            ic_cdk::println!("[send_signed_transactions] tx_hash = {transaction_hash:?}");
            transaction_hash.ok_or_else(|| BlockchainError::EmptyTransactionHash.into())
        }
        SendRawTransactionStatus::NonceTooLow => Err(BlockchainError::NonceTooLow)?,
        SendRawTransactionStatus::NonceTooHigh => Err(BlockchainError::NonceTooHigh)?,
        SendRawTransactionStatus::InsufficientFunds => Err(BlockchainError::InsufficientFunds)?,
    }
}

async fn send_raw_transaction(tx: String, chain_id: u64) -> Result<SendRawTransactionStatus> {
    let rpc_providers = get_rpc_providers(chain_id).unwrap();
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
                SendRawTransactionResult::Ok(status) => Ok(status),
                SendRawTransactionResult::Err(RpcError::JsonRpcError(e)) => {
                    if e.message.contains("nonce too low") {
                        Ok(SendRawTransactionStatus::NonceTooLow)
                    } else if e.message.contains("nonce too high") {
                        Ok(SendRawTransactionStatus::NonceTooHigh)
                    } else if e.message.contains("insufficient funds") {
                        Ok(SendRawTransactionStatus::InsufficientFunds)
                    } else {
                        Err(SystemError::RpcError(format!("Json Rpc Error: {:?}", e)).into())
                    }
                }
                SendRawTransactionResult::Err(e) => {
                    Err(SystemError::RpcError(format!("{:?}", e)).into())
                }
            },
            MultiSendRawTransactionResult::Inconsistent(_) => {
                Err(BlockchainError::InconsistentStatus.into())
            }
        },
        Err((code, msg)) => Err(SystemError::ICRejectionError(code, msg).into()),
    }
}

pub async fn check_transaction_status(tx_hash: &String, chain_id: u64) -> TransactionStatus {
    let cycles = 10_000_000_000;
    let rpc_providers = get_rpc_providers(chain_id).unwrap();
    let arg: Option<RpcConfig> = None;
    let res = EVM_RPC
        .eth_get_transaction_receipt(rpc_providers, arg, tx_hash.to_string(), cycles)
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

pub fn spawn_transaction_checker<F, G>(
    retry_attempt: u8,
    tx_hash: String,
    chain_id: u64,
    order_id: u64,
    sign_request: SignRequest,
    on_success: F,
    on_fail: G,
) where
    F: Fn(TransactionReceipt) + 'static,
    G: Fn() + 'static,
{
    fn schedule_check<F, G>(
        tx_hash: String,
        chain_id: u64,
        attempt: u16,
        retry_attempt: u8,
        order_id: u64,
        sign_request: SignRequest,
        on_success: F,
        on_fail: G,
    ) where
        F: Fn(TransactionReceipt) + 'static,
        G: Fn() + 'static,
    {
        ic_cdk_timers::set_timer(Duration::from_secs(ATTEMPT_INTERVAL_SECONDS), move || {
            ic_cdk::println!("[schedule_check] spawning attempt number: {}", attempt);
            ic_cdk::spawn(async move {
                match check_transaction_status(&tx_hash, chain_id).await {
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
                            on_fail,
                        );
                    }
                    TransactionStatus::Pending if attempt == MAX_ATTEMPTS_PER_RETRY - 1 => {
                        ic_cdk::spawn(async move {
                            retry_with_bumped_fees(
                                sign_request,
                                tx_hash,
                                chain_id,
                                order_id,
                                retry_attempt,
                                on_success,
                                on_fail,
                            )
                            .await;
                        });
                    }
                    TransactionStatus::Failed(err) => {
                        ic_cdk::println!("[schedule_check] TransactionStatus::Failed: {:?}", err);
                        on_fail();
                        logs::update_transaction_log(order_id, TransactionStatus::Failed(err));
                    }
                    _ => {
                        ic_cdk::println!(
                            "[schedule_check] Transaction status check exceeded maximum attempts"
                        );
                        on_fail();
                        logs::remove_transaction_log(order_id);
                    }
                }
            });
        });
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
            on_fail,
        );
    } else if retry_attempt == MAX_RETRY_ATTEMPTS {
        on_fail();
        ic_cdk::spawn(async move {
            match bump_dummy_transaction(sign_request.clone(), chain_id).await {
                Ok(tx_hash) => ic_cdk::println!("[bump_dummy_transaction] tx_hash = {}", tx_hash),
                Err(e) => {
                    ic_cdk::println!("[bump_dummy_transaction] failed: {}", e);
                    logs::update_transaction_log(
                        order_id,
                        TransactionStatus::Unresolved(tx_hash, sign_request.into()),
                    );
                }
            };
        })
    }
}

pub async fn retry_with_bumped_fees<F, G>(
    mut sign_request: SignRequest,
    last_tx_hash: String,
    chain_id: u64,
    order_id: u64,
    retry_attempt: u8,
    on_success: F,
    on_fail: G,
) where
    F: Fn(TransactionReceipt) + 'static,
    G: Fn() + 'static,
{
    // Bump gas fees
    sign_request.max_fee_per_gas = Some(bump_fee(sign_request.max_fee_per_gas));
    sign_request.max_priority_fee_per_gas = Some(bump_fee(sign_request.max_priority_fee_per_gas));

    ic_cdk::println!(
        "[retry_with_bumped_fees] attempt number: {}. New sign request max fee {:?}, max priority: {:?}",
        retry_attempt,
        sign_request.max_fee_per_gas.map(|fee| fee.as_u128()),
        sign_request.max_priority_fee_per_gas.map(|fee| fee.as_u128())
    );

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
                sign_request,
                on_success,
                on_fail,
            );
        }
        Err(e) => {
            ic_cdk::println!(
                "[retry_with_bumped_fees] Failed to send transaction: {:?}",
                e
            );
            on_fail();
            match bump_dummy_transaction(sign_request.clone(), chain_id).await {
                Ok(tx_hash) => ic_cdk::println!("[bump_dummy_transaction] tx_hash = {}", tx_hash),
                Err(e) => {
                    ic_cdk::println!("[bump_dummy_transaction] failed: {}", e);
                    logs::update_transaction_log(
                        order_id,
                        TransactionStatus::Unresolved(last_tx_hash, sign_request.into()),
                    );
                }
            };
        }
    }
}

fn bump_fee(current_fee: Option<U256>) -> U256 {
    // Bump gas fee 20%
    current_fee.unwrap_or(U256::zero()) * U256::from(12) / U256::from(10)
}

pub async fn bump_dummy_transaction(sign_request: SignRequest, chain_id: u64) -> Result<String> {
    let sign_request: SignRequest = SignRequest {
        data: None,
        value: Some(U256::zero()),
        max_fee_per_gas: Some(bump_fee(sign_request.max_fee_per_gas)),
        max_priority_fee_per_gas: Some(bump_fee(sign_request.max_priority_fee_per_gas)),
        ..sign_request
    };

    send_signed_transaction(sign_request.clone(), chain_id).await
}

pub async fn eth_get_transaction_count(chain_id: u64) -> Result<u128> {
    let rpc_providers = chains::get_rpc_providers(chain_id)?;
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
                GetTransactionCountResult::Err(e) => {
                    Err(SystemError::RpcError(format!("{:?}", e)))?
                }
            },
            MultiGetTransactionCountResult::Inconsistent(_) => {
                ic_cdk::trap("Block Result is inconsistent");
            }
        },
        Err((code, message)) => Err(SystemError::ICRejectionError(code, message))?,
    }
}
