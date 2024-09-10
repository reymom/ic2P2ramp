use std::collections::HashMap;

use crate::evm::{fees, vault::Ic2P2ramp};
use crate::management::user as user_management;
use crate::model::helpers;
use crate::types::{
    calculate_fees, chains,
    order::{Order, OrderFilter, OrderState, OrderStateFilter},
    Blockchain, PaymentProvider, PaymentProviderType, TransactionAddress,
};
use crate::{
    errors::{RampError, Result},
    state, storage,
};

pub async fn create_order(
    offramper_user_id: u64,
    fiat_amount: u64,
    currency_symbol: String,
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    blockchain: Blockchain,
    token: Option<String>,
    crypto_amount: u128,
    offramper_address: TransactionAddress,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u64> {
    let (offramper_fee, crypto_fee) = match blockchain {
        Blockchain::EVM { chain_id } => {
            let estimated_gas_lock = estimated_gas_lock.ok_or_else(|| {
                RampError::InvalidInput(
                    "Gas estimation for locking is required for EVM".to_string(),
                )
            })?;
            let estimated_gas_withdraw = estimated_gas_withdraw.ok_or_else(|| {
                RampError::InvalidInput(
                    "Gas estimation for withdrawing is required for EVM".to_string(),
                )
            })?;

            let total_gas_estimation = Ic2P2ramp::get_final_gas(estimated_gas_lock)
                + Ic2P2ramp::get_final_gas(estimated_gas_withdraw);
            ic_cdk::println!(
                "[create_order] total_gas_estimation = {:?}",
                total_gas_estimation
            );

            let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
            ic_cdk::println!("[create_order] fee_estimates = {:?}", fee_estimates);

            let mut blockchain_fees =
                total_gas_estimation as u128 * fee_estimates.max_fee_per_gas.as_u128();
            ic_cdk::println!("[create_order] blockchain_fees = {:?}", blockchain_fees);
            if let Some(token_address) = token.clone() {
                let xrc_symbol = chains::get_evm_token_symbol(chain_id, &token_address)?;
                let rate = helpers::get_eth_token_rate(xrc_symbol).await?;
                ic_cdk::println!("[create_order] token rate = {:?}", rate);

                blockchain_fees = (blockchain_fees as f64 * rate) as u128;
                ic_cdk::println!(
                    "[create_order] blockchain_fees after = {:?}",
                    blockchain_fees
                );
            }
            calculate_fees(fiat_amount, crypto_amount, blockchain_fees)
        }
        Blockchain::ICP { ledger_principal } => {
            let icp_fee: u128 = state::get_fee(&ledger_principal)?
                .0
                .try_into()
                .map_err(|e| {
                    RampError::InternalError(format!(
                        "icp fee cannot be converted to u128: {:?}",
                        e
                    ))
                })?;

            calculate_fees(fiat_amount, crypto_amount, icp_fee * 2)
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    };

    if crypto_fee >= crypto_amount {
        return Err(RampError::FundsBelowFees);
    }

    let order = Order::new(
        offramper_user_id,
        fiat_amount,
        currency_symbol,
        offramper_providers,
        blockchain,
        token,
        crypto_amount,
        offramper_address,
        offramper_fee,
        crypto_fee,
    )?;

    storage::insert_order(&order);
    Ok(order.id)
}

pub fn get_orders(
    filter: Option<OrderFilter>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Vec<OrderState> {
    match filter {
        None => storage::ORDERS.with(|p| {
            let start_index = page.unwrap_or(1).saturating_sub(1) * page_size.unwrap_or(10);
            let end_index = start_index + page_size.unwrap_or(10);

            p.borrow()
                .iter()
                .skip(start_index as usize)
                .take((end_index - start_index) as usize)
                .map(|(_, v)| v.clone())
                .collect()
        }),
        Some(OrderFilter::ByOfframperId(offramper_id)) => storage::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.offramper_user_id == offramper_id,
                OrderState::Locked(order) => order.base.offramper_user_id == offramper_id,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByOnramperId(onramper_id)) => storage::filter_orders(
            |order_state| match order_state {
                OrderState::Locked(order) => order.onramper_user_id == onramper_id,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByOfframperAddress(address)) => storage::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.offramper_address == address,
                OrderState::Locked(order) => order.base.offramper_address == address,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::LockedByOnramper(address)) => storage::filter_orders(
            |order_state| match order_state {
                OrderState::Locked(order) => order.onramper_address == address,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByState(state)) => storage::filter_orders(
            |order_state| match (state.clone(), order_state) {
                (OrderStateFilter::Created, OrderState::Created(_))
                | (OrderStateFilter::Locked, OrderState::Locked(_))
                | (OrderStateFilter::Completed, OrderState::Completed(_))
                | (OrderStateFilter::Cancelled, OrderState::Cancelled(_)) => true,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByBlockchain(blockchain)) => storage::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.crypto.blockchain == blockchain,
                OrderState::Locked(order) => order.base.crypto.blockchain == blockchain,
                _ => false,
            },
            page,
            page_size,
        ),
    }
}

pub fn lock_order(
    order_id: u64,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    consent_id: Option<String>,
    consent_url: Option<String>,
) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Created(order) => {
            *order_state = OrderState::Locked(order.clone().lock(
                onramper_user_id,
                onramper_provider,
                onramper_address,
                consent_id,
                consent_url,
            )?);
            Ok(())
        }
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    })??;

    state::set_order_timer(order_id);

    Ok(())
}

pub fn unlock_order(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            user_management::decrease_user_score(order.onramper_user_id)?;
            ic_cdk::println!("[unlock_order] user score decreased");
            *order_state = OrderState::Created(order.base.clone());
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })??;

    state::clear_order_timer(order_id)
}

pub fn mark_order_as_paid(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            user_management::update_onramper_payment(
                order.onramper_user_id,
                order.base.fiat_amount,
            )?;
            user_management::update_offramper_payment(
                order.base.offramper_user_id,
                order.base.fiat_amount,
            )?;
            order.payment_done = true;
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })??;

    state::clear_order_timer(order_id)
}

pub fn set_payment_id(order_id: u64, payment_id: String) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            order.payment_id = Some(payment_id);
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn cancel_order(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Created(_) => {
            *order_state = OrderState::Cancelled(order_id);
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn set_order_completed(order_id: u64) -> Result<()> {
    storage::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            *order_state = OrderState::Completed(order.clone().complete());
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}
