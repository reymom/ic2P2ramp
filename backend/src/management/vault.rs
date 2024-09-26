use num_traits::ToPrimitive;

use crate::{
    evm::transaction,
    model::memory,
    types::{
        evm::{
            gas::{self, MethodGasUsage},
            logs::TransactionAction,
            request::SignRequest,
        },
        orders::RevolutConsent,
        PaymentProvider, TransactionAddress,
    },
};

use super::on_fail_callback;

pub(super) fn spawn_commit_listener(
    order_id: u64,
    chain_id: u64,
    price: u64,
    offramper_fee: u64,
    tx_hash: &str,
    sign_request: SignRequest,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    revolut_consent: Option<RevolutConsent>,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        Some(TransactionAction::Commit),
        sign_request.clone(),
        move |receipt| {
            let gas_used = receipt.gasUsed.0.to_u128().unwrap_or(0);
            let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
            let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

            if !(gas_used == 0 || gas_price == 0) {
                match gas::register_gas_usage(
                    chain_id,
                    gas_used,
                    gas_price,
                    block_number,
                    &MethodGasUsage::Commit,
                ) {
                    Ok(()) => ic_cdk::println!(
                        "[lock_order].[register_gas_usage] Gas Used: {}, Gas Price: {}, Block Number: {}",
                        gas_used,
                        gas_price,
                        block_number
                    ),
                    Err(err) => ic_cdk::println!("[lock_order].[register_gas_usage] error: {:?}", err),
                }
            } else {
                ic_cdk::println!(
                    "[lock_order] Gas Used: {}, Gas Price: {}",
                    gas_used,
                    gas_price
                );
            }

            match memory::stable::orders::lock_order(
                order_id,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider.clone(),
                onramper_address.clone(),
                revolut_consent.clone(),
            ) {
                Ok(()) => ic_cdk::println!("order {:?} is locked!", order_id),
                Err(err) => {
                    ic_cdk::println!("order {:?} failed to be locked: {:?}", order_id, err)
                }
            };
        },
        on_fail_callback(order_id),
    );
}

pub(super) fn spawn_uncommit_listener(
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
        Some(TransactionAction::Uncommit),
        sign_request,
        move |receipt| {
            let gas_used = receipt.gasUsed.0.to_u128().unwrap_or(0);
            let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
            let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

            ic_cdk::println!(
                "[internal_unlock_order] Gas Used: {}, Gas Price: {}, Block Number: {}",
                gas_used,
                gas_price,
                block_number
            );

            // Unlock the order in the backend once the transaction succeeds
            if let Err(e) = memory::stable::orders::unlock_order(order_id) {
                ic_cdk::println!(
                    "[unlock_order] failed to unlock order #{:?}, error: {:?}",
                    order_id,
                    e
                );
            };
        },
        on_fail_callback(order_id),
    );
}

pub(super) fn spawn_cancel_order(
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
        Some(TransactionAction::Cancel),
        sign_request,
        move |receipt| {
            let gas_used = receipt.gasUsed.0.to_u128().unwrap_or(0);
            let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
            let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

            ic_cdk::println!(
                "[cancel_order] Gas Used: {}, Gas Price: {}, Block Number: {}",
                gas_used,
                gas_price,
                block_number
            );

            // Cancel the order in the backend once the transaction succeeds
            match memory::stable::orders::cancel_order(order_id) {
                Ok(_) => {
                    ic_cdk::println!("order {:?} is cancelled!", order_id);
                }
                Err(e) => {
                    ic_cdk::println!(
                        "[cancel_order] failed to cancel order #{:?}, error: {:?}",
                        order_id,
                        e
                    );
                }
            }
        },
        on_fail_callback(order_id),
    );
}

pub(super) fn spawn_payment_release(
    order_id: u64,
    chain_id: u64,
    action_type: MethodGasUsage,
    tx_hash: &str,
    sign_request: SignRequest,
) {
    transaction::spawn_transaction_checker(
        0,
        tx_hash.to_string(),
        chain_id,
        order_id,
        Some(TransactionAction::Release),
        sign_request,
        move |receipt| {
            let gas_used = receipt.gasUsed.0.to_u128().unwrap_or(0);
            let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
            let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

            ic_cdk::println!(
                "[verify_transaction] Gas Used: {}, Gas Price: {}, Block Number: {}",
                gas_used,
                gas_price,
                block_number
            );

            if !(gas_used == 0 || gas_price == 0) {
                let _ = gas::register_gas_usage(
                    chain_id,
                    gas_used,
                    gas_price,
                    block_number,
                    &action_type,
                );
            }

            // Update order state to completed
            match super::order::set_order_completed(order_id) {
                Ok(_) => {
                    ic_cdk::println!("[verify_transaction] order {:?} completed", order_id)
                }
                Err(e) => ic_cdk::trap(format!("could not complete order: {:?}", e).as_str()),
            }
        },
        super::on_fail_callback(order_id),
    );
}
