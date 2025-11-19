use candid::Principal;
use evm_rpc_canister_types::LogEntry;
use icramp_types::bitcoin::{runes::RuneUTXOEntry, transfer::TransactionType};
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use crate::errors::{BlockchainError, OrderError, Result, SystemError, UserError};
use crate::inter_canister::bitcoin::{
    bitcoin_backend_estimate_fee, bitcoin_backend_get_rune_metadata, bitcoin_backend_lock_funds,
    bitcoin_backend_transfer, bitcoin_backend_unlock_funds,
};
use crate::inter_canister::solana::{
    solana_backend_estimate_fees, solana_backend_get_token_info, solana_backend_lock_funds,
    solana_backend_send_sol, solana_backend_send_spl_token, solana_backend_unlock_funds,
};
use crate::management::bitcoin as bitcoin_management;
use crate::management::solana::{SolanaTransactionAction, spawn_solana_tx_listener};
use crate::management::user as user_management;
use crate::{
    evm::{
        event::{self, LogEvent},
        fees::get_fee_estimates,
        transaction,
        vault::Ic2P2ramp,
    },
    outcalls::stripe::session::create_checkout_session_for_order,
};
use crate::{icp::vault::Ic2P2ramp as ICPRamp, model::memory::heap::clear_order_timer};

use crate::model::{
    guards, helpers,
    memory::{self, stable::spent_transactions},
};
use crate::outcalls::pricing::rates::get_exchange_rate;
use crate::types::{
    self, BlockchainAsset, PaymentProvider, PaymentProviderType, TransactionAddress,
    evm::{logs::TransactionStatus, token},
    exchange_rate::RateAsset,
    find_provider_of_type,
    icp::get_icp_token,
    orders::{
        LockedOrder, Order, OrderFilter, OrderState, OrderStateFilter,
        fees::{get_crypto_fee, get_fiat_fee},
    },
};

use super::payment;

const MIN_REMAINING_MINOR: u64 = 500; // avoid dust (<$5) in partial fills

pub async fn calculate_price_and_fee(
    currency: &str,
    asset: &BlockchainAsset,
    amount: u128,
) -> Result<(u64, u64)> {
    let (symbol, decimals) = asset.get_symbol_and_decimals().await?;
    let base_asset = asset.to_rate_asset(&symbol);
    let quote_asset = RateAsset::Fiat {
        symbol: currency.to_string(),
    };

    let exchange_rate = get_exchange_rate(base_asset, quote_asset).await?;
    let crypto_units = (amount as f64) / (10u128.pow(decimals as u32) as f64);
    let fiat_amount = (crypto_units * exchange_rate * 100.) as u64;

    Ok((fiat_amount, get_fiat_fee(fiat_amount)))
}

pub async fn calculate_order_evm_fees(
    chain_id: u64,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_withdraw: u64,
) -> Result<u128> {
    let total_gas_estimation = Ic2P2ramp::get_final_gas(estimated_gas_withdraw);
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

pub async fn order_crypto_fee(
    asset: BlockchainAsset,
    crypto_amount: u128,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u128> {
    match asset.clone() {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            let estimated_gas_withdraw = estimated_gas_withdraw.ok_or_else(|| {
                SystemError::InvalidInput(
                    "Gas estimation for withdrawing is required for EVM".to_string(),
                )
            })?;

            calculate_order_evm_fees(
                chain_id,
                crypto_amount,
                token_address.clone(),
                estimated_gas_withdraw,
            )
            .await
        }
        BlockchainAsset::ICP { ledger_principal } => {
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
        BlockchainAsset::Bitcoin { rune_id } => {
            let mut fee = bitcoin_backend_estimate_fee().await?;
            if let Some(rune_id) = rune_id {
                let metadata = bitcoin_backend_get_rune_metadata(rune_id.to_string()).await?;
                let rate = helpers::get_btc_token_rate(metadata.name).await?;
                let scale_factor = 10u128.pow(metadata.divisibility as u32);
                fee = (fee as f64 * rate * scale_factor as f64) as u64;
            }
            Ok(get_crypto_fee(crypto_amount, fee.saturating_mul(2) as u128))
        }
        BlockchainAsset::Solana { spl_token } => {
            let fees = solana_backend_estimate_fees(spl_token.clone()).await?;
            let lamports_total = (fees.lock_lamports as u128) + (fees.withdraw_lamports as u128);

            // If native SOL: fees are already in base units (lamports).
            if spl_token.is_none() {
                return Ok(get_crypto_fee(crypto_amount, lamports_total));
            }

            // SPL: convert SOL fees (lamports) → token base units, using SOL→token rate and token decimals.
            let mint = spl_token.clone().unwrap();
            let info = solana_backend_get_token_info(mint.clone()).await?;

            // SOL → TOKEN (human units), then → base units by decimals
            let rate = helpers::get_sol_token_rate(info.rate_symbol, spl_token).await?;
            let sol_fees = (lamports_total as f64) / 1_000_000_000f64;
            let token_fees_human = sol_fees * rate;
            let scale = 10u128.pow(info.decimals as u32) as f64;

            // ceil to avoid undercharging
            let token_fees_base: u128 = (token_fees_human * scale).ceil() as u128;

            ic_cdk::println!(
                "[order_crypto_fee] sol-fees -> lock_lamports={}, withdraw_lamports={}, lamports_total={}, rate(tokens/SOL)≈{}, decimals={}, token_fee_human≈{}, token_fee_base={}",
                fees.lock_lamports,
                fees.withdraw_lamports,
                lamports_total,
                rate,
                info.decimals,
                token_fees_human,
                token_fees_base
            );
            Ok(get_crypto_fee(crypto_amount, token_fees_base))
        }
    }
}

pub async fn get_valid_log_event(chain_id: &u64, tx_hash: &String) -> Result<(LogEvent, LogEntry)> {
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
            Ok((event::parse_log_event(log_entry)?, log_entry.clone()))
        }
        _ => Err(BlockchainError::EmptyTransactionHash.into()),
    }
}

pub async fn create_order(
    currency: &str,
    offramper_user_id: u64,
    offramper_address: TransactionAddress,
    offramper_providers: Vec<PaymentProvider>,
    asset: BlockchainAsset,
    crypto_amount: u128,
    estimated_gas_withdraw: Option<u64>,
    runes: Option<Vec<RuneUTXOEntry>>,
) -> Result<u64> {
    let crypto_fee = order_crypto_fee(asset.clone(), crypto_amount, estimated_gas_withdraw).await?;

    ic_cdk::println!(
        "[create_order] crypto_amount = {:?}, crypto_fee = {:?}",
        crypto_amount,
        crypto_fee
    );

    if 2 * crypto_fee >= crypto_amount {
        return Err(BlockchainError::FundsTooLow)?;
    }

    let order = Order::new(
        currency.to_string(),
        offramper_user_id,
        offramper_address,
        offramper_providers,
        asset,
        crypto_amount,
        crypto_fee,
        runes,
    )?;

    memory::stable::orders::insert_order(&order);
    Ok(order.id)
}

pub async fn topup_order(
    order: &Order,
    amount: u128,
    estimated_gas_withdraw: Option<u64>,
) -> Result<()> {
    let new_total = order.crypto.amount + amount;
    let crypto_fee = order_crypto_fee(
        order.crypto.asset.clone(),
        new_total,
        estimated_gas_withdraw,
    )
    .await?;

    if 2 * crypto_fee >= new_total {
        return Err(BlockchainError::FundsTooLow)?;
    };

    memory::stable::orders::mutate_order(&order.id, |order| {
        let order = order.created_mut()?;
        order.crypto.amount = new_total;
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
                .map(|e| e.value())
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
        Some(OrderFilter::ByBlockchain(blockchain_type)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => {
                    order.crypto.asset.blockchain_type() == blockchain_type
                }
                OrderState::Locked(order) => {
                    order.base.crypto.asset.blockchain_type() == blockchain_type
                }
                _ => false,
            },
            page,
            page_size,
        ),
        Some(OrderFilter::ByBlockchainAsset(asset)) => memory::stable::orders::filter_orders(
            |order_state| match order_state {
                OrderState::Created(order) => order.crypto.asset == asset,
                OrderState::Locked(order) => order.base.crypto.asset == asset,
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
    lock_amount: u128,
    stripe_success_url: Option<String>,
    stripe_cancel_url: Option<String>,
) -> Result<()> {
    let user = memory::stable::users::get_user(&onramper_user_id)?;
    user.validate_session(&session_token)?;
    user.validate_onramper()?;
    user.is_banned()?;

    let order = memory::stable::orders::get_order(&order_id)?.created()?;

    // filling amount checks
    if lock_amount == 0 || lock_amount > order.crypto.amount {
        return Err(OrderError::InvalidInput("invalid partial amount".into()).into());
    }
    let remaining = order.crypto.amount - lock_amount;
    if remaining > 0 {
        let (rem_minor, _) =
            calculate_price_and_fee(&order.currency, &order.crypto.asset, remaining).await?;
        if rem_minor < MIN_REMAINING_MINOR {
            return Err(OrderError::InvalidInput(
                "remaining below minimum; lock full amount".into(),
            )
            .into());
        }
    }

    // provider checks
    let off_supports_stripe = order
        .offramper_providers
        .iter()
        .any(|p| matches!(p.provider_type(), PaymentProviderType::Stripe));
    let on_is_email = onramper_provider.provider_type() == PaymentProviderType::Email;

    let provider_ok = if on_is_email {
        off_supports_stripe && user.payment_providers.contains(&onramper_provider)
    } else {
        types::contains_provider_type(&onramper_provider, &order.offramper_providers)
    };
    if !provider_ok {
        return Err(OrderError::InvalidOnramperProvider.into());
    }

    // price/fee for filling amount
    let (price, offramper_fee) =
        calculate_price_and_fee(&order.currency, &order.crypto.asset, lock_amount).await?;

    // (fee must be < 50% of fiat price):
    if offramper_fee.saturating_mul(2) >= price {
        return Err(BlockchainError::FundsTooLow.into());
    }

    let revolut_consent = payment::revolut::get_revolut_consent(
        order.offramper_providers.clone(),
        &(price as f64 / 100.).to_string(),
        &order.currency,
        &onramper_provider,
    )
    .await?;

    let stripe_session: Option<(String, String)> = if on_is_email {
        let (acct, platform) = order
            .offramper_providers
            .iter()
            .find_map(|p| {
                if let PaymentProvider::Stripe {
                    account_id,
                    platform,
                } = p
                {
                    Some((account_id.clone(), platform.clone()))
                } else {
                    None
                }
            })
            .ok_or(OrderError::InvalidOnramperProvider)?;
        let payer_email = if let PaymentProvider::Email { email } = onramper_provider.clone() {
            email
        } else {
            return Err(OrderError::InvalidOnramperProvider.into());
        };
        let (sid, url) = create_checkout_session_for_order(
            order_id,
            &acct,
            price + offramper_fee,
            &order.currency,
            Some(platform),
            stripe_success_url.ok_or_else(|| {
                OrderError::InvalidInput("Stripe success_url should be present".to_string())
            })?,
            stripe_cancel_url.ok_or_else(|| {
                OrderError::InvalidInput("Stripe cancel_url should be present".to_string())
            })?,
            payer_email,
        )
        .await?;
        Some((sid, url))
    } else {
        None
    };

    match order.crypto.asset {
        BlockchainAsset::EVM { .. } | BlockchainAsset::ICP { .. } => {
            memory::stable::orders::lock_order(
                order_id,
                lock_amount,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address,
                revolut_consent,
                stripe_session,
            )?;

            Ok(())
        }
        BlockchainAsset::Bitcoin { rune_id } => {
            memory::stable::orders::lock_order(
                order_id,
                lock_amount,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address.clone(),
                revolut_consent,
                stripe_session,
            )?;
            bitcoin_backend_lock_funds(
                order.offramper_address.address,
                onramper_address.address,
                lock_amount as u64,
                rune_id,
            )
            .await?;
            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            memory::stable::orders::lock_order(
                order_id,
                lock_amount,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address.clone(),
                revolut_consent,
                stripe_session,
            )?;
            solana_backend_lock_funds(
                order.offramper_address.address,
                onramper_address.address,
                lock_amount as u64,
                spl_token,
            )
            .await?;
            Ok(())
        }
    }
}

/// Unlocks an order, handling both ICP, EVM and Bitcoin orders.
///
/// # Parameters
///
/// - `order_id`: The unique identifier of the order to be unlocked.
///
/// # Behavior
///
/// - **ICP and EVM Orders**: Unlocks the order directly.
/// - **Bitcoin Orders**: Unlocks the order and calls the `bitcoin_backend` canister
///   to update the vault tracking state.
/// - **Solana orders**: Unlocks the order both in here and mirrors it in `solana_backend`.
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
///   or any transaction fails.
///
/// # Example
/// ```
/// let result = unlock_order(12345).await;
/// match result {
///     Ok(()) => println!("Transaction succeeded."),
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

    match order.base.crypto.asset {
        BlockchainAsset::EVM { .. } | BlockchainAsset::ICP { .. } => {
            memory::stable::orders::unlock_order(order.base.id)?;
            Ok(())
        }
        BlockchainAsset::Bitcoin { rune_id } => {
            memory::stable::orders::unlock_order(order.base.id)?;

            bitcoin_backend_unlock_funds(
                order.base.offramper_address.address,
                order.onramper.address.address,
                order.base.crypto.amount as u64,
                rune_id,
            )
            .await?;

            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            memory::stable::orders::unlock_order(order.base.id)?;

            solana_backend_unlock_funds(
                order.base.offramper_address.address,
                order.onramper.address.address,
                order.base.crypto.amount as u64,
                spl_token,
            )
            .await?;

            Ok(())
        }
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

    match &order.crypto.asset {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            let fees = order.crypto.fee / 2;
            Ic2P2ramp::withdraw_deposit(
                *chain_id,
                order_id,
                order.offramper_address.address,
                token_address.clone(),
                order.crypto.amount,
                fees,
            )
            .await?;
            Ok(())
        }
        BlockchainAsset::ICP { ledger_principal } => {
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
        BlockchainAsset::Bitcoin { rune_id } => {
            let dst_address = order.offramper_address.address.clone();
            let tx_type = match rune_id.clone() {
                Some(rune_id) => TransactionType::RuneTransfer(rune_id),
                None => TransactionType::TaprootBitcoin,
            };
            let tx_id = bitcoin_backend_transfer(
                dst_address.clone(),
                order.crypto.amount as u64,
                tx_type,
                order.crypto.rune_utxos,
            )
            .await?;

            bitcoin_management::spawn_bitcoin_tx_listener(
                tx_id,
                bitcoin_management::BitcoinTransactionAction::CancelOrder {
                    order_id,
                    amount: order.crypto.amount as u64,
                    offramper_address: dst_address.clone(),
                },
                dst_address,
                rune_id.clone(),
                0,
            );

            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            let to = order.offramper_address.address.clone();
            let amt_nat = candid::Nat::from(order.crypto.amount);

            // 1. Refund
            let sig = match &spl_token {
                Some(mint) => {
                    solana_backend_send_spl_token(mint.clone(), to.clone(), amt_nat).await?
                }
                None => {
                    solana_backend_send_sol(to.clone(), candid::Nat::from(order.crypto.amount))
                        .await?
                }
            };

            // 2. Cancel order after confirmation
            spawn_solana_tx_listener(
                sig,
                SolanaTransactionAction::CancelOrder {
                    order_id,
                    amount: order.crypto.amount as u64,
                    offramper: to,
                    token: spl_token.clone(),
                },
                0,
            );

            Ok(())
        }
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

    clear_order_timer(order_id)
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

    let off_supports_stripe = order
        .base
        .offramper_providers
        .iter()
        .any(|p| matches!(p.provider_type(), PaymentProviderType::Stripe));
    let on_is_email = matches!(order.onramper.provider, PaymentProvider::Email { .. });

    if !(off_supports_stripe && on_is_email) {
        find_provider_of_type(
            &order.base.offramper_providers,
            order.onramper.provider.provider_type(),
        )
        .ok_or_else(|| UserError::ProviderNotInUser(order.onramper.provider.provider_type()))?;
    }

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
