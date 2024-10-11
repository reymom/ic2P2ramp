use std::collections::HashMap;

use candid::Principal;
use evm_rpc_canister_types::BlockTag;
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::NumTokens;

use crate::errors::{BlockchainError, OrderError, Result, SystemError, UserError};
use crate::evm::{
    event::{self, LogEvent},
    fees::{eth_get_latest_block, get_fee_estimates},
    transaction,
    vault::Ic2P2ramp,
};
use crate::icp::vault::Ic2P2ramp as ICPRamp;
use crate::management::user as user_management;
use crate::model::guards;
use crate::model::{
    helpers,
    memory::{self, stable::spent_transactions},
};
use crate::outcalls::xrc_rates::{get_cached_exchange_rate, Asset, AssetClass};
use crate::types::{
    self,
    evm::{chains, logs::TransactionStatus, token, transaction::TransactionAction},
    icp::{get_icp_token, is_icp_token_supported},
    orders::{
        fees::{get_crypto_fee, get_fiat_fee},
        EvmOrderInput, LockInput, LockedOrder, Order, OrderFilter, OrderState, OrderStateFilter,
    },
    Blockchain, Crypto, PaymentProvider, PaymentProviderType, TransactionAddress,
};

use super::payment;

pub async fn calculate_price_and_fee(currency: &str, crypto: &Crypto) -> Result<(u64, u64)> {
    let base_asset = Asset {
        class: AssetClass::Cryptocurrency,
        symbol: crypto.get_symbol()?,
    };
    let quote_asset = Asset {
        class: AssetClass::FiatCurrency,
        symbol: currency.to_string(),
    };
    let exchange_rate = get_cached_exchange_rate(base_asset, quote_asset).await?;

    let fiat_amount = (crypto.to_whole_units()? * exchange_rate * 100.) as u64;

    Ok((fiat_amount, get_fiat_fee(fiat_amount)))
}

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

    let fee_estimates = get_fee_estimates(9, chain_id).await?;
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

async fn order_crypto_fee(
    blockchain: Blockchain,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u128> {
    match blockchain {
        Blockchain::EVM { chain_id } => {
            let estimated_gas_lock = estimated_gas_lock.ok_or_else(|| {
                SystemError::InvalidInput(
                    "Gas estimation for locking is required for EVM".to_string(),
                )
            })?;
            let estimated_gas_withdraw = estimated_gas_withdraw.ok_or_else(|| {
                SystemError::InvalidInput(
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
            .await
        }
        Blockchain::ICP { ledger_principal } => {
            let icp_fee: u128 =
                get_icp_token(&ledger_principal)?
                    .fee
                    .0
                    .try_into()
                    .map_err(|e| {
                        SystemError::InternalError(format!(
                            "icp fee cannot be converted to u128: {:?}",
                            e
                        ))
                    })?;

            Ok(get_crypto_fee(crypto_amount, icp_fee * 2))
        }
        _ => Err(BlockchainError::UnsupportedBlockchain)?,
    }
}

pub async fn get_valid_log_event(chain_id: &u64, tx_hash: &String) -> Result<LogEvent> {
    if spent_transactions::is_tx_hash_processed(tx_hash) {
        return Err(
            BlockchainError::EvmLogError("Transaction already processed".to_string()).into(),
        );
    };

    match transaction::check_transaction_status(tx_hash, *chain_id).await {
        TransactionStatus::Confirmed(receipt) => {
            let log_entry = receipt
                .logs
                .first()
                .ok_or_else(|| BlockchainError::EvmLogError("Empty Log Entries".to_string()))?;
            event::parse_deposit_event(log_entry)
        }
        _ => Err(BlockchainError::EmptyTransactionHash.into()),
    }
}

pub async fn validate_deposit_tx(
    blockchain: &Blockchain,
    evm_input: Option<EvmOrderInput>,
    order_offramper: String,
    order_amount: u128,
    order_token: Option<String>,
) -> Result<Option<String>> {
    match blockchain {
        Blockchain::EVM { chain_id } => {
            chains::chain_is_supported(*chain_id)?;
            if let Some(token) = order_token.clone() {
                token::evm_token_is_approved(*chain_id, &token)?;
            };

            let evm_input = evm_input.ok_or_else(|| {
                BlockchainError::EvmLogError("EVM input data is required".to_string())
            })?;

            let log_event = get_valid_log_event(chain_id, &evm_input.tx_hash).await?;
            ic_cdk::println!("[validate_deposit_tx] log_event = {:?}", log_event);
            match log_event {
                LogEvent::Deposit(deposit_event) => {
                    if deposit_event.user.to_lowercase() != order_offramper.to_lowercase() {
                        return Err(BlockchainError::EvmLogError(
                            "Invalid Offramper Address".to_string(),
                        )
                        .into());
                    };
                    if deposit_event.amount != order_amount {
                        return Err(BlockchainError::EvmLogError(
                            "Invalid Crypto Amount".to_string(),
                        )
                        .into());
                    }
                    if deposit_event.token.clone().map(|t| t.to_lowercase())
                        != order_token.map(|t| t.to_lowercase())
                    {
                        return Err(
                            BlockchainError::EvmLogError("Invalid Crypto".to_string()).into()
                        );
                    }

                    let last_block = eth_get_latest_block(*chain_id, BlockTag::Latest)
                        .await
                        .map(|block| block.number)?;
                    deposit_event.expired(last_block)?;
                }
            };

            Ok(Some(evm_input.tx_hash))
        }
        Blockchain::ICP { ledger_principal } => {
            is_icp_token_supported(ledger_principal)?;
            Ok(None)
        }
        _ => Err(BlockchainError::UnsupportedBlockchain)?,
    }
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
    let crypto_fee = order_crypto_fee(
        blockchain.clone(),
        crypto_amount,
        token.clone(),
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await?;

    if 2 * crypto_fee >= crypto_amount {
        return Err(BlockchainError::FundsTooLow)?;
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

pub async fn topup_order(
    order: &Order,
    amount: u128,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<()> {
    let crypto_fee = order_crypto_fee(
        order.crypto.blockchain.clone(),
        order.crypto.amount,
        order.crypto.token.clone(),
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await?;

    if 2 * crypto_fee >= order.crypto.amount + amount {
        return Err(BlockchainError::FundsTooLow)?;
    };

    memory::stable::orders::mutate_order(&order.id, |order| {
        let order = order.created_mut()?;
        order.crypto.amount += amount;
        order.crypto.fee = crypto_fee;
        Ok(())
    })?
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
            |order_state| {
                matches!(
                    (state.clone(), order_state),
                    (OrderStateFilter::Created, OrderState::Created(_))
                        | (OrderStateFilter::Locked, OrderState::Locked(_))
                        | (OrderStateFilter::Completed, OrderState::Completed(_))
                        | (OrderStateFilter::Cancelled, OrderState::Cancelled(_))
                )
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
    session_token: String,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
) -> Result<()> {
    let user = memory::stable::users::get_user(&onramper_user_id)?;
    user.validate_session(&session_token)?;
    user.validate_onramper()?;
    user.is_banned()?;

    let order = memory::stable::orders::get_order(&order_id)?.created()?;

    if !types::contains_provider_type(&onramper_provider, &order.offramper_providers) {
        return Err(OrderError::InvalidOnramperProvider)?;
    }

    let (price, offramper_fee) = calculate_price_and_fee(&order.currency, &order.crypto).await?;

    let revolut_consent = payment::get_revolut_consent(
        order.offramper_providers,
        &(price as f64 / 100.).to_string(),
        &order.currency,
        &onramper_provider,
    )
    .await?;

    match order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let estimated_gas =
                Ic2P2ramp::get_average_gas_price(chain_id, &TransactionAction::Commit).await?;
            Ic2P2ramp::commit_deposit(
                chain_id,
                order_id,
                order.offramper_address.address,
                order.crypto.token,
                order.crypto.amount,
                Some(estimated_gas),
                LockInput {
                    price,
                    offramper_fee,
                    onramper_user_id,
                    onramper_provider,
                    onramper_address,
                    revolut_consent,
                },
            )
            .await?;
            Ok(())
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
            Ok(())
        }
        _ => Err(BlockchainError::UnsupportedBlockchain)?,
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
pub async fn unlock_order(order_id: u64) -> Result<()> {
    let order = memory::stable::orders::get_order(&order_id)?.locked()?;
    if order.payment_done {
        return Err(OrderError::PaymentDone)?;
    }
    if order.uncommited {
        return Err(OrderError::OrderUncommitted)?;
    }
    if order.is_inside_lock_time() {
        return Err(OrderError::OrderInLockTime)?;
    }

    let user = memory::stable::users::get_user(&order.onramper.user_id)?;
    user.validate_onramper()?;

    match order.base.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let estimated_gas =
                Ic2P2ramp::get_average_gas_price(chain_id, &TransactionAction::Uncommit).await?;
            Ic2P2ramp::uncommit_deposit(
                chain_id,
                order_id,
                order.base.offramper_address.address,
                order.base.crypto.token,
                order.base.crypto.amount,
                Some(estimated_gas),
            )
            .await?;
            Ok(())
        }
        Blockchain::ICP { .. } => {
            memory::stable::orders::unlock_order(order.base.id)?;
            Ok(())
        }
        _ => Err(BlockchainError::UnsupportedBlockchain)?,
    }
}

pub async fn cancel_order(order_id: u64, session_token: String) -> Result<()> {
    let order = memory::stable::orders::get_order(&order_id)?.created()?;
    let user = memory::stable::users::get_user(&order.offramper_user_id)?;
    user.is_offramper()?;
    if !user.addresses.contains(&order.offramper_address) {
        return Err(UserError::Unauthorized.into());
    }
    user.validate_session(&session_token)?;

    match &order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let fees = order.crypto.fee / 2;
            Ic2P2ramp::withdraw_deposit(
                *chain_id,
                order_id,
                order.offramper_address.address,
                order.crypto.token,
                order.crypto.amount,
                fees,
            )
            .await?;
            Ok(())
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
            Ok(())
        }
        _ => Err(BlockchainError::UnsupportedBlockchain)?,
    }
}

pub fn mark_order_as_paid(order_id: u64) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| -> Result<()> {
        match order_state {
            OrderState::Locked(order) => {
                user_management::update_onramper_payment(
                    order.onramper.user_id,
                    order.price,
                    &order.base.currency,
                )?;
                user_management::update_offramper_payment(
                    order.base.offramper_user_id,
                    order.price,
                    &order.base.currency,
                )?;
                order.payment_done = true;
                Ok(())
            }
            _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
        }
    })??;

    memory::heap::clear_order_timer(order_id)
}

pub fn set_payment_id(order_id: u64, payment_id: String) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            order.payment_id = Some(payment_id);
            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })?
}

pub fn set_order_completed(order_id: u64) -> Result<()> {
    memory::stable::orders::mutate_order(&order_id, |order_state| match order_state {
        OrderState::Locked(order) => {
            *order_state = OrderState::Completed(order.clone().complete());
            Ok(())
        }
        _ => Err(OrderError::InvalidOrderState(order_state.to_string()))?,
    })?
}

pub fn verify_order_is_payable(
    order_id: u64,
    session_token: Option<String>,
) -> Result<LockedOrder> {
    let order = memory::stable::orders::get_order(&order_id)?.locked()?;
    if !order.is_inside_lock_time() {
        Err(OrderError::OrderUncommitted)?;
    }
    if order.payment_done {
        Err(OrderError::PaymentDone)?;
    };
    if order.uncommited {
        Err(OrderError::OrderUncommitted)?;
    }
    order
        .base
        .offramper_providers
        .get(&order.onramper.provider.provider_type())
        .ok_or_else(|| UserError::ProviderNotInUser(order.onramper.provider.provider_type()))?;

    let user = memory::stable::users::get_user(&order.onramper.user_id)?;
    if let Some(session_token) = session_token {
        user.validate_session(&session_token)?;
        user.is_banned()?;
    } else {
        guards::only_controller()?;
    }

    user.validate_onramper()?;

    Ok(order)
}
