use std::collections::HashMap;

use candid::Principal;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::evm::{fees, vault::Ic2P2ramp};
use crate::icp::vault::Ic2P2ramp as ICPRamp;
use crate::management::user as user_management;
use crate::model::memory::stable;
use crate::model::types;
use crate::types::{
    evm::token,
    get_icp_fee,
    order::{calculate_fees, LockedOrder, Order, OrderFilter, OrderState, OrderStateFilter},
    Blockchain, PaymentProvider, PaymentProviderType, TransactionAddress,
};
use crate::{
    errors::{RampError, Result},
    model::{helpers, memory},
};

use super::{payment, vault};

pub async fn calculate_order_evm_fees(
    chain_id: u64,
    fiat_amount: u64,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_lock: u64,
    estimated_gas_withdraw: u64,
) -> Result<(u64, u128)> {
    let total_gas_estimation = Ic2P2ramp::get_final_gas(estimated_gas_lock)
        + Ic2P2ramp::get_final_gas(estimated_gas_withdraw);
    ic_cdk::println!(
        "[calculate_order_evm_fees] total_gas_estimation = {:?}",
        total_gas_estimation
    );

    let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
    ic_cdk::println!(
        "[calculate_order_evm_fees] fee_estimates = {:?}",
        fee_estimates
    );

    let mut blockchain_fees =
        total_gas_estimation as u128 * fee_estimates.max_fee_per_gas.as_u128();
    ic_cdk::println!(
        "[calculate_order_evm_fees] blockchain_fees = {:?}",
        blockchain_fees
    );

    if let Some(token_address) = token {
        let token = token::get_evm_token(chain_id, &token_address)?;
        let rate = helpers::get_eth_token_rate(token.rate_symbol).await?;
        ic_cdk::println!("[calculate_order_evm_fees] token rate = {:?}", rate);

        let scale_factor = 10u128.pow(18 - token.decimals as u32);
        blockchain_fees = ((blockchain_fees as f64 * rate) / scale_factor as f64) as u128;
        ic_cdk::println!(
            "[calculate_order_evm_fees] blockchain_fees after = {:?}",
            blockchain_fees
        );
    }

    Ok(calculate_fees(fiat_amount, crypto_amount, blockchain_fees))
}

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

            calculate_order_evm_fees(
                chain_id,
                fiat_amount,
                crypto_amount,
                token.clone(),
                estimated_gas_lock,
                estimated_gas_withdraw,
            )
            .await?
        }
        Blockchain::ICP { ledger_principal } => {
            let icp_fee: u128 = get_icp_fee(&ledger_principal)?.0.try_into().map_err(|e| {
                RampError::InternalError(format!("icp fee cannot be converted to u128: {:?}", e))
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

    memory::stable::orders::insert_order(&order);
    Ok(order.id)
}

pub fn get_orders(
    filter: Option<OrderFilter>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Vec<OrderState> {
    match filter {
        None => memory::stable::storage::ORDERS.with(|p| {
            let start_index = page.unwrap_or(1).saturating_sub(1) * page_size.unwrap_or(10);
            let end_index = start_index + page_size.unwrap_or(10);

            p.borrow()
                .iter()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .skip(start_index as usize)
                .take((end_index - start_index) as usize)
                .map(|(_, v)| v.clone())
                .collect()
        }),
        Some(OrderFilter::ByOfframperId(offramper_id)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.offramper_user_id == offramper_id,
                OrderState::Locked(order) => order.base.offramper_user_id == offramper_id,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByOnramperId(onramper_id)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Locked(order) => order.onramper_user_id == onramper_id,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByOfframperAddress(address)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.offramper_address == address,
                OrderState::Locked(order) => order.base.offramper_address == address,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::LockedByOnramper(address)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Locked(order) => order.onramper_address == address,
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByState(state)) => memory::stable::orders::filter_orders(
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
        Some(OrderFilter::ByBlockchain(blockchain)) => memory::stable::orders::filter_orders(
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

pub async fn lock_order(
    order_id: u64,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    estimated_gas: Option<u64>,
) -> Result<String> {
    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Created(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    if !types::contains_provider_type(&onramper_provider, &order.offramper_providers) {
        return Err(RampError::InvalidOnramperProvider);
    }

    let (revolut_consent_id, consent_url) =
        payment::get_revolut_consent(&order, &onramper_provider).await?;

    match order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let tx_hash = Ic2P2ramp::commit_deposit(
                chain_id,
                order.offramper_address.address,
                order.crypto.token,
                order.crypto.amount,
                estimated_gas,
            )
            .await?;

            // Listener for the transaction receipt
            vault::spawn_commit_listener(
                order_id,
                chain_id,
                &tx_hash,
                onramper_user_id,
                onramper_provider,
                onramper_address,
                revolut_consent_id,
                consent_url,
            );

            return Ok(tx_hash);
        }
        Blockchain::ICP { .. } => {
            memory::stable::orders::lock_order(
                order_id,
                onramper_user_id,
                onramper_provider,
                onramper_address,
                revolut_consent_id,
                consent_url,
            )?;

            return Ok(format!("order {:?} is locked!", order_id));
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
}

// unlock_order manages the logic for the unlock of icp orders.
// In the case of EVM orders, first it uncommits the funds in the vault,
// and listens for the transaction until it completes before updating the ICP order.
// session token is optional, if it is internally called, it should be set to None.
pub async fn unlock_order(
    order_id: u64,
    session_token: Option<String>,
    estimated_gas: Option<u64>,
) -> Result<String> {
    let order_state = memory::stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };
    if order.payment_done {
        return Err(RampError::PaymentDone);
    }

    if let Some(session_token) = session_token {
        let user = stable::users::get_user(&order.onramper_user_id)?;
        user.validate_session(&session_token)?;
        user.validate_onramper()?;
    }

    match order.base.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            // Uncommit the deposit in the EVM vault
            let tx_hash = Ic2P2ramp::uncommit_deposit(
                chain_id,
                order.base.offramper_address.address,
                order.base.crypto.token,
                order.base.crypto.amount,
                estimated_gas,
            )
            .await?;

            // Listener for the transaction receipt
            vault::spawn_uncommit_listener(order_id, chain_id, &tx_hash);

            Ok(tx_hash)
        }
        Blockchain::ICP { ledger_principal } => {
            memory::stable::orders::unlock_order(order.base.id)?;
            Ok(format!(
                "Unlocked ICP order for ledger: {:?}",
                ledger_principal.to_string()
            ))
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
}

pub async fn cancel_order(order_id: u64, session_token: String) -> Result<String> {
    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Created(order) => order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let user = stable::users::get_user(&order.offramper_user_id)?;
    user.is_offramper()?;
    user.validate_session(&session_token)?;

    match &order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let fees = order.crypto.fee / 2;

            // let sign_request: SignRequest;
            let tx_hash = if let Some(token_address) = order.crypto.token {
                Ic2P2ramp::withdraw_token(
                    *chain_id,
                    order.offramper_address.address.clone(),
                    token_address.clone(),
                    order.crypto.amount,
                    fees,
                )
                .await?
            } else {
                Ic2P2ramp::withdraw_base_currency(
                    *chain_id,
                    order.offramper_address.address.clone(),
                    order.crypto.amount,
                    fees,
                )
                .await?
            };

            vault::spawn_cancel_order(order_id, *chain_id, &tx_hash);

            Ok(tx_hash)
        }
        Blockchain::ICP { ledger_principal } => {
            let offramper_principal =
                Principal::from_text(&order.offramper_address.address).unwrap();

            let amount = NumTokens::from(order.crypto.amount);
            let fee = get_icp_fee(ledger_principal)?;

            let to_account = Account {
                owner: offramper_principal,
                subaccount: None,
            };
            ic_cdk::println!("[cancel] amount = {}, fee: {}", amount, fee);
            ICPRamp::transfer(
                *ledger_principal,
                to_account,
                amount - fee.clone(),
                Some(fee),
            )
            .await?;

            memory::stable::orders::cancel_order(order_id)?;
            Ok(format!(
                "Cancelled ICP order for ledger: {:?}",
                ledger_principal.to_string()
            ))
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
}

pub fn mark_order_as_paid(order_id: u64) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| match order_state {
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

    memory::heap::clear_order_timer(order_id)
}

pub fn set_payment_id(order_id: u64, payment_id: String) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            order.payment_id = Some(payment_id);
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn set_order_completed(order_id: u64) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            *order_state = OrderState::Completed(order.clone().complete());
            Ok(())
        }
        _ => Err(RampError::InvalidOrderState(order_state.to_string())),
    })?
}

pub fn verify_order_is_payable(order_id: u64, session_token: &str) -> Result<LockedOrder> {
    let order_state = memory::stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };
    if order.payment_done {
        return Err(RampError::PaymentDone);
    };
    if order.uncommited {
        return Err(RampError::OrderUncommitted);
    }
    order
        .base
        .offramper_providers
        .get(&order.onramper_provider.provider_type())
        .ok_or_else(|| RampError::ProviderNotInUser(order.onramper_provider.provider_type()))?;

    let user = memory::stable::users::get_user(&order.onramper_user_id)?;
    user.validate_session(session_token)?;
    user.is_banned()?;
    user.validate_onramper()?;

    Ok(order)
}
