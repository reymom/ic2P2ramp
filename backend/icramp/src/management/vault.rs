use evm_rpc_canister_types::TransactionReceipt;
use num_traits::ToPrimitive;

use crate::{
    evm::transaction,
    model::memory,
    types::{
        evm::{
            gas,
            request::SignRequest,
            transaction::{TransactionAction, TransactionVariant},
        },
        orders::LockInput,
    },
};

use super::on_fail_callback;

fn register_gas_usage(
    chain_id: u64,
    receipt: &TransactionReceipt,
    action_type: &TransactionAction,
) {
    ic_cdk::println!(
        "[vault].[register_gas_usage] Chain_id: {}, action_type: {:?}",
        chain_id,
        action_type
    );
    let gas_used = receipt.gasUsed.0.to_u64().unwrap_or(0);
    let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
    let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

    if gas_used != 0 && block_number != 0 {
        match gas::register_gas_usage(chain_id, gas_used, gas_price, block_number, action_type) {
            Ok(()) => ic_cdk::println!(
                "[vault].[register_gas_usage] Gas Used: {}, Gas Price: {}, Block Number: {}",
                gas_used,
                gas_price,
                block_number
            ),
            Err(err) => ic_cdk::println!("[vault].[register_gas_usage] error: {:?}", err),
        }
    } else {
        ic_cdk::println!(
            "[vault].[register_gas_usage] gas_used: {}, block_number: {}",
            gas_used,
            block_number
        );
    }
}

pub fn spawn_commit_listener(
    order_id: u64,
    chain_id: u64,
    tx_hash: &str,
    sign_request: SignRequest,
    lock_input: LockInput,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        sign_request.clone(),
        move |receipt| {
            register_gas_usage(chain_id, &receipt, &TransactionAction::Commit);

            // Lock the order in the storage once the transaction succeeds
            match memory::stable::orders::lock_order(
                order_id,
                lock_input.price,
                lock_input.offramper_fee,
                lock_input.onramper_user_id,
                lock_input.onramper_provider.clone(),
                lock_input.onramper_address.clone(),
                lock_input.revolut_consent.clone(),
            ) {
                Ok(()) => ic_cdk::println!("[commit] order {} is locked.", order_id),
                Err(err) => {
                    ic_cdk::println!("[commit] order {} failed to be locked: {:?}", order_id, err)
                }
            };
        },
        on_fail_callback(order_id),
    );
}

pub fn spawn_uncommit_listener(
    order_id: u64,
    chain_id: u64,
    tx_hash: &str,
    sign_request: SignRequest,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        sign_request,
        move |receipt| {
            register_gas_usage(chain_id, &receipt, &TransactionAction::Uncommit);

            // Unlock the order in the storage once the transaction succeeds
            match memory::stable::orders::unlock_order(order_id) {
                Ok(()) => ic_cdk::println!("[uncommit] order {} is unlocked.", order_id),
                Err(e) => ic_cdk::println!(
                    "[uncommit] failed to unlock order #{:?}, error: {:?}",
                    order_id,
                    e
                ),
            };
        },
        on_fail_callback(order_id),
    );
}

pub fn spawn_cancel_listener(
    order_id: u64,
    chain_id: u64,
    cancel_variant: TransactionVariant,
    tx_hash: &str,
    sign_request: SignRequest,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        sign_request,
        move |receipt| {
            register_gas_usage(
                chain_id,
                &receipt,
                &TransactionAction::Cancel(cancel_variant.clone()),
            );

            // Cancel the order in the backend once the transaction succeeds
            match memory::stable::orders::cancel_order(order_id) {
                Ok(()) => ic_cdk::println!("[withdraw] order {:?} is cancelled!", order_id),
                Err(e) => ic_cdk::println!(
                    "[withdraw] failed to cancel order #{:?}, error: {:?}",
                    order_id,
                    e
                ),
            }
        },
        on_fail_callback(order_id),
    );
}

pub fn spawn_release_listener(
    order_id: u64,
    chain_id: u64,
    release_variant: TransactionVariant,
    tx_hash: &str,
    sign_request: SignRequest,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        sign_request,
        move |receipt| {
            register_gas_usage(
                chain_id,
                &receipt,
                &TransactionAction::Release(release_variant.clone()),
            );

            // Update order state to completed
            match super::order::set_order_completed(order_id) {
                Ok(()) => ic_cdk::println!("[release_funds] order {} is completed.", order_id),
                Err(e) => ic_cdk::println!(
                    "[relese_funds] could not complete order: {}, error: {:?}",
                    order_id,
                    e
                ),
            }
        },
        super::on_fail_callback(order_id),
    );
}
