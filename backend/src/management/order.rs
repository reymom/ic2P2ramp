use std::collections::HashMap;

use candid::Principal;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::evm::{fees, vault::Ic2P2ramp};
use crate::icp::vault::Ic2P2ramp as ICPRamp;
use crate::management::user as user_management;
use crate::model::memory::stable;
use crate::model::types::evm::chains;
use crate::model::types::icp::get_icp_token;
use crate::model::types::order::get_fiat_fee;
use crate::model::types::{self, icp};
use crate::outcalls::xrc_rates::{get_cached_exchange_rate, Asset, AssetClass};
use crate::types::{
    evm::token,
    order::{get_crypto_fee, LockedOrder, Order, OrderFilter, OrderState, OrderStateFilter},
    Blockchain, PaymentProvider, PaymentProviderType, TransactionAddress,
};
use crate::{
    errors::{RampError, Result},
    model::{helpers, memory},
};

use super::{payment, vault};

pub async fn calculate_order_evm_fees(
    chain_id: u64,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_lock: u64,
    estimated_gas_withdraw: u64,
) -> Result<u128> {
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

    Ok(get_crypto_fee(crypto_amount, blockchain_fees))
}

pub async fn create_order(
    currency: &str,
    offramper_user_id: u64,
    offramper_address: TransactionAddress,
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    blockchain: Blockchain,
    token: Option<String>,
    crypto_amount: u128,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u64> {
    let crypto_fee = match blockchain {
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
                crypto_amount,
                token.clone(),
                estimated_gas_lock,
                estimated_gas_withdraw,
            )
            .await?
        }
        Blockchain::ICP { ledger_principal } => {
            let icp_fee: u128 =
                get_icp_token(&ledger_principal)?
                    .fee
                    .0
                    .try_into()
                    .map_err(|e| {
                        RampError::InternalError(format!(
                            "icp fee cannot be converted to u128: {:?}",
                            e
                        ))
                    })?;

            get_crypto_fee(crypto_amount, icp_fee * 2)
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    };

    if crypto_fee >= crypto_amount {
        return Err(RampError::FundsBelowFees);
    }

    let order = Order::new(
        currency.to_string(),
        offramper_user_id,
        offramper_address,
        offramper_providers,
        blockchain,
        token,
        crypto_amount,
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
                OrderState::Locked(order) => order.onramper.user_id == onramper_id,
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
                OrderState::Locked(order) => order.onramper.address == address,
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

pub async fn calculate_price_and_fee(
    currency: &str,
    crypto_amount: u128,
    crypto_symbol: &str,
) -> Result<(u64, u64)> {
    let base_asset = Asset {
        class: AssetClass::Cryptocurrency,
        symbol: crypto_symbol.to_string(),
    };
    let quote_asset = Asset {
        class: AssetClass::FiatCurrency,
        symbol: currency.to_string(),
    };
    let exchange_rate = get_cached_exchange_rate(base_asset, quote_asset).await?;

    let fiat_amount = (crypto_amount as f64 * exchange_rate * 100.) as u64;
    let fiat_fee = get_fiat_fee(fiat_amount);

    Ok((fiat_amount, fiat_fee))
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

    let crypto_symbol = match order.crypto.blockchain {
        Blockchain::EVM { chain_id } => chains::get_currency_symbol(chain_id),
        Blockchain::ICP { ledger_principal } => Ok(icp::get_icp_token(&ledger_principal)?.symbol),
        _ => return Err(RampError::UnsupportedBlockchain),
    }?;

    let (price, offramper_fee) =
        calculate_price_and_fee(&order.currency, order.crypto.amount, &crypto_symbol).await?;

    let revolut_consent = payment::get_revolut_consent(
        order.offramper_providers,
        &(price as f64 / 100.).to_string(),
        &order.currency,
        &onramper_provider,
    )
    .await?;

    match order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let (tx_hash, sign_request) = Ic2P2ramp::commit_deposit(
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
                price,
                offramper_fee,
                &tx_hash,
                sign_request,
                onramper_user_id,
                onramper_provider,
                onramper_address,
                revolut_consent,
            );

            return Ok(tx_hash);
        }
        Blockchain::ICP { .. } => {
            memory::stable::orders::lock_order(
                order_id,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address,
                revolut_consent,
            )?;

            return Ok(format!("order {:?} is locked!", order_id));
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
}

/// Unlocks an order, handling both ICP and EVM blockchain orders.
///
/// # Parameters
///
/// - `order_id`: The unique identifier of the order to be unlocked.
/// - `session_token`: An optional session token used to validate the request.
///    If the function is called internally, this can be set to `None`.
/// - `estimated_gas`: An optional estimated gas limit for EVM transactions.
///    This is only applicable for EVM orders.
///
/// # Behavior
///
/// - **ICP Orders**: Unlocks the order directly.
/// - **EVM Orders**: First, uncommits the funds in the EVM vault. The function
///   listens for the EVM transaction to complete successfully before proceeding
///   to update the corresponding ICP order status.
///
/// # Returns
///
/// - On success: Returns a `String` representing the transaction hash or a
///   confirmation message.
/// - On failure: Returns a `RampError` with details about why the unlock failed.
///
/// # Errors
///
/// - Returns an error if the order cannot be found, the session is invalid,
///   or if the EVM transaction fails.
///
/// # Example
/// ```
/// let result = unlock_order(12345, Some("session_token"), Some(21000)).await;
/// match result {
///     Ok(tx_hash) => println!("Transaction succeeded with hash: {}", tx_hash),
///     Err(err) => eprintln!("Failed to unlock order: {:?}", err),
/// }
/// ```
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
        let user = stable::users::get_user(&order.onramper.user_id)?;
        user.validate_session(&session_token)?;
        user.validate_onramper()?;
    }

    match order.base.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            // Uncommit the deposit in the EVM vault
            let (tx_hash, sign_request) = Ic2P2ramp::uncommit_deposit(
                chain_id,
                order.base.offramper_address.address,
                order.base.crypto.token,
                order.base.crypto.amount,
                estimated_gas,
            )
            .await?;

            // Listener for the transaction receipt
            vault::spawn_uncommit_listener(order_id, chain_id, &tx_hash, sign_request);

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
            let (tx_hash, sign_request) = if let Some(token_address) = order.crypto.token {
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

            vault::spawn_cancel_order(order_id, *chain_id, &tx_hash, sign_request);

            Ok(tx_hash)
        }
        Blockchain::ICP { ledger_principal } => {
            let offramper_principal =
                Principal::from_text(&order.offramper_address.address).unwrap();

            let amount = NumTokens::from(order.crypto.amount);
            let fee = get_icp_token(ledger_principal)?.fee;

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
            user_management::update_onramper_payment(order.onramper.user_id, order.price)?;
            user_management::update_offramper_payment(order.base.offramper_user_id, order.price)?;
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
        .get(&order.onramper.provider.provider_type())
        .ok_or_else(|| RampError::ProviderNotInUser(order.onramper.provider.provider_type()))?;

    let user = memory::stable::users::get_user(&order.onramper.user_id)?;
    user.validate_session(session_token)?;
    user.is_banned()?;
    user.validate_onramper()?;

    Ok(order)
}
