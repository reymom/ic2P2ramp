use evm_rpc_canister_types::TransactionReceipt;
use num_traits::ToPrimitive;

use crate::{
    evm::transaction,
    model::memory::{
        self,
        stable::orders::{finalize_pending_fill, set_order_completed},
    },
    types::evm::{
        gas,
        request::SignRequest,
        transaction::{TransactionAction, TransactionVariant},
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
            let _ = finalize_pending_fill(order_id, Some(receipt.transactionHash.to_string()));
            match set_order_completed(order_id) {
                Ok(()) => ic_cdk::println!("[release_funds] order lock {} filled", order_id),
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
