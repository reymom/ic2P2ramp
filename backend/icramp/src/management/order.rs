use std::collections::HashMap;

use candid::Principal;
use evm_rpc_canister_types::BlockTag;
use icramp_types::{
    bitcoin::{runes::RuneUTXOEntry, transfer::TransactionType},
    solana::errors::{SolanaError, TransactionError},
};
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
use crate::inter_canister::bitcoin::{
    bitcoin_backend_estimate_fee, bitcoin_backend_get_rune_metadata, bitcoin_backend_lock_funds,
    bitcoin_backend_transfer, bitcoin_backend_unlock_funds, bitcoin_backend_validate_rune,
};
use crate::inter_canister::solana::{
    solana_backend_estimate_fees, solana_backend_get_token_info, solana_backend_get_tx_metadata,
    solana_backend_lock_funds, solana_backend_send_sol, solana_backend_send_spl_token,
    solana_backend_unlock_funds,
};
use crate::management::bitcoin as bitcoin_management;
use crate::management::solana::{SolanaTransactionAction, spawn_solana_tx_listener};
use crate::management::user as user_management;

use crate::model::{
    guards, helpers,
    memory::{self, stable::spent_transactions},
};
use crate::outcalls::pricing::rates::get_exchange_rate;
use crate::types::{
    self, BlockchainAsset, Crypto, PaymentProvider, PaymentProviderType, TransactionAddress,
    evm::{chains, logs::TransactionStatus, token, transaction::TransactionAction},
    exchange_rate::RateAsset,
    icp::{get_icp_token, is_icp_token_supported},
    orders::{
        DepositInput, LockInput, LockedOrder, Order, OrderFilter, OrderState, OrderStateFilter,
        fees::{get_crypto_fee, get_fiat_fee},
    },
};

use super::payment;

pub async fn calculate_price_and_fee(currency: &str, crypto: &Crypto) -> Result<(u64, u64)> {
    let base_symbol = crypto.asset.get_symbol().await?;
    let base_asset = crypto.asset.to_rate_asset(&base_symbol);
    let quote_asset = RateAsset::Fiat {
        symbol: currency.to_string(),
    };

    let exchange_rate = get_exchange_rate(base_asset, quote_asset).await?;
    let fiat_amount = (crypto.to_whole_units().await? * exchange_rate * 100.) as u64;

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
    asset: BlockchainAsset,
    crypto_amount: u128,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u128> {
    match asset.clone() {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
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
                token_address.clone(),
                estimated_gas_lock,
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
            Ok(get_crypto_fee(crypto_amount, fee as u128))
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

            Ok(get_crypto_fee(crypto_amount, token_fees_base))
        }
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
    asset: &BlockchainAsset,
    deposit_input: Option<DepositInput>,
    order_offramper: String,
    order_amount: u128,
) -> Result<Option<String>> {
    match asset {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            chains::chain_is_supported(*chain_id)?;
            if let Some(token) = token_address.clone() {
                token::evm_token_is_approved(*chain_id, &token)?;
            };

            let evm_input = match deposit_input {
                Some(DepositInput::Evm(v)) => Ok(v),
                _ => Err(OrderError::InvalidInput(
                    "Missing evm order input".to_string(),
                )),
            }?;

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
                        != token_address.clone().map(|t| t.to_lowercase())
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
        BlockchainAsset::ICP { ledger_principal } => {
            is_icp_token_supported(ledger_principal)?;
            Ok(None)
        }
        BlockchainAsset::Bitcoin { rune_id } => {
            if let Some(rune_id) = rune_id.clone() {
                bitcoin_backend_validate_rune(rune_id).await?;
            };

            let bitcoin_input = match deposit_input {
                Some(DepositInput::Bitcoin(v)) => Ok(v),
                _ => Err(OrderError::InvalidInput(
                    "Missing bitcoin order input".to_string(),
                )),
            }?;

            let bitcoin_txid = bitcoin_input.tx_id;
            if spent_transactions::is_tx_hash_processed(&bitcoin_txid) {
                return Err(BlockchainError::BitcoinBackendError(
                    "Transaction already processed".to_string(),
                )
                .into());
            };

            Ok(Some(bitcoin_txid))
        }
        BlockchainAsset::Solana {
            spl_token: expected_mint,
        } => {
            let sol_input = match deposit_input {
                Some(DepositInput::Solana(v)) => Ok(v),
                _ => Err(OrderError::InvalidInput(
                    "Missing solana order input".to_string(),
                )),
            }?;

            if spent_transactions::is_tx_hash_processed(&sol_input.signature) {
                return Err(BlockchainError::TransactionAlreadyProcessed.into());
            }

            // Optional local input sanity vs asset
            match (&expected_mint, &sol_input.mint) {
                (Some(exp), Some(got)) if !exp.eq_ignore_ascii_case(got) => {
                    return Err(OrderError::InvalidInput("SPL mint mismatch".to_string()).into());
                }
                (None, Some(_)) => {
                    return Err(OrderError::InvalidInput(
                        "Unexpected SPL mint for SOL deposit".to_string(),
                    )
                    .into());
                }
                _ => {}
            }

            let meta = solana_backend_get_tx_metadata(sol_input.signature.clone())
                .await?
                .meta;

            if let Some(exp_mint) = expected_mint {
                let mut pre: HashMap<(u8, String), u128> = HashMap::new();
                if let Some(pre_tbs) = meta.pre_token_balances.as_ref() {
                    for tb in pre_tbs {
                        if tb.mint.eq_ignore_ascii_case(exp_mint) {
                            if let (i, Some(a)) = (
                                tb.account_index,
                                tb.ui_token_amount.amount.parse::<u128>().ok(),
                            ) {
                                pre.insert((i, tb.mint.clone()), a);
                            }
                        }
                    }
                }

                let mut hits = 0usize;
                if let Some(post_tbs) = meta.post_token_balances.as_ref() {
                    for tb in post_tbs {
                        if tb.mint.eq_ignore_ascii_case(exp_mint) {
                            if let (i, Some(post_amt)) = (
                                tb.account_index,
                                tb.ui_token_amount.amount.parse::<u128>().ok(),
                            ) {
                                let k = (i, tb.mint.clone());
                                let pre_amt = pre.get(&k).copied().unwrap_or(0);
                                let delta = post_amt.saturating_sub(pre_amt);
                                if delta == order_amount {
                                    hits += 1;
                                } else if delta > 0 {
                                    return Err(SolanaError::from(TransactionError::MetaError(
                                        "ambiguous SPL credits in tx".to_string(),
                                    ))
                                    .into());
                                }
                            }
                        }
                    }
                }

                if hits != 1 {
                    return Err(SolanaError::from(TransactionError::MetaError(
                        "SPL deposit not found / ambiguous".to_string(),
                    ))
                    .into());
                }
            } else {
                // -------- SOL VALIDATION (best-effort without account keys) --------
                // We can’t map indices→addresses here without parsed keys. Validate that there exists
                // a **single** positive lamports delta equal to order_amount.
                let pre = &meta.pre_balances;
                let post = &meta.post_balances;
                if pre.len() != post.len() {
                    return Err(SolanaError::from(TransactionError::MetaError(
                        "invalid lamport vectors".to_string(),
                    ))
                    .into());
                }

                let mut matches = 0usize;
                for (a, b) in pre.iter().zip(post.iter()) {
                    let delta = b.saturating_sub(*a) as u128;
                    if delta == order_amount {
                        matches += 1;
                    } else if delta > 0 && delta != order_amount {
                        // Another credit in same tx → ambiguous
                        return Err(SolanaError::from(TransactionError::MetaError(
                            "ambiguous SOL credits in tx".to_string(),
                        ))
                        .into());
                    }
                }
                if matches != 1 {
                    return Err(SolanaError::from(TransactionError::MetaError(
                        "SOL deposit not found / ambiguous".to_string(),
                    ))
                    .into());
                }
            }

            Ok(Some(sol_input.signature))
        }
    }
}

pub async fn create_order(
    currency: &str,
    offramper_user_id: u64,
    offramper_address: TransactionAddress,
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    asset: BlockchainAsset,
    crypto_amount: u128,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
    runes: Option<Vec<RuneUTXOEntry>>,
) -> Result<u64> {
    let crypto_fee = order_crypto_fee(
        asset.clone(),
        crypto_amount,
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await?;

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
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<()> {
    let crypto_fee = order_crypto_fee(
        order.crypto.asset.clone(),
        order.crypto.amount,
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

    match order.crypto.asset {
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            let estimated_gas =
                Ic2P2ramp::get_average_gas_price(chain_id, &TransactionAction::Commit).await?;
            Ic2P2ramp::commit_deposit(
                chain_id,
                order_id,
                order.offramper_address.address,
                token_address,
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
        BlockchainAsset::ICP { .. } => {
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
        BlockchainAsset::Bitcoin { rune_id } => {
            memory::stable::orders::lock_order(
                order_id,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address.clone(),
                revolut_consent,
            )?;

            bitcoin_backend_lock_funds(
                order.offramper_address.address,
                onramper_address.address,
                order.crypto.amount as u64,
                rune_id,
            )
            .await?;
            Ok(())
        }
        BlockchainAsset::Solana { spl_token } => {
            memory::stable::orders::lock_order(
                order_id,
                price,
                offramper_fee,
                onramper_user_id,
                onramper_provider,
                onramper_address.clone(),
                revolut_consent,
            )?;

            solana_backend_lock_funds(
                order.offramper_address.address,
                onramper_address.address,
                order.crypto.amount as u64,
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
/// - **ICP Orders**: Unlocks the order directly.
/// - **Bitcoin Orders**: Unlocks the order and calls the `bitcoin_backend` canister
///   to update the vault tracking state.
/// - **Solana orders**: Unlocks the order both in here and mirrors it in `solana_backend`.
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
        BlockchainAsset::EVM {
            chain_id,
            token_address,
        } => {
            let estimated_gas =
                Ic2P2ramp::get_average_gas_price(chain_id, &TransactionAction::Uncommit).await?;
            Ic2P2ramp::uncommit_deposit(
                chain_id,
                order_id,
                order.base.offramper_address.address,
                token_address,
                order.base.crypto.amount,
                Some(estimated_gas),
            )
            .await?;
            Ok(())
        }
        BlockchainAsset::ICP { .. } => {
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
