mod debug;
mod evm;
mod icp;
mod inter_canister;
mod management;
mod model;
mod outcalls;

use std::collections::{HashMap, HashSet};

use candid::Principal;
use evm_rpc_canister_types::BlockTag;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use icramp_types::bitcoin::runes::RuneID;
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use evm::{fees, transaction, vault::Ic2P2ramp};
use icp::vault::Ic2P2ramp as ICPRamp;
use inter_canister::solana::solana_backend_deposit_funds;
use management::bitcoin::BitcoinTransactionAction;
use management::{
    bitcoin as bitcoin_management, order as order_management, payment as payment_management,
    random, user as user_management,
};
use model::errors::{self, BlockchainError, OrderError, Result, SystemError, UserError};
use model::types::{
    self, AddressType, AuthenticationData, BlockchainAsset, LoginAddress, PaymentProvider,
    PaymentProviderType, TransactionAddress,
    evm::{
        gas::{self, ChainGasTracking},
        logs::{EvmTransactionLog, TransactionStatus},
        nonce,
        token::{self, Token, TokenManager},
        transaction::{TransactionAction, TransactionVariant},
    },
    exchange_rate::{CACHE_DURATION, ExchangeRateCache, RateAsset},
    icp::{IcpToken, get_icp_token},
    orders::fees::{FeeQuote, get_admin_fee},
    orders::{
        BitcoinOrderInput, DepositInput, EvmOrderInput, OrderFilter, OrderState, SolanaOrderInput,
    },
    session::Session,
    stripe::StripeAccountInfo,
    user::{User, UserType},
};
use model::{
    guards, helpers,
    memory::{
        self,
        heap::{
            self, InstallArg, STATE, State, get_state, initialize_state, logs, read_state,
            setup_timers, upgrade,
        },
        stable::{self, orders, spent_transactions},
    },
};
use outcalls::{
    pricing::rates,
    revolut::token as revolut_token,
    stripe::account::{create_account_link, create_express_account, get_account_info},
};

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    upgrade::pre_upgrade()
}

#[ic_cdk::post_upgrade]
fn post_upgrade(install_arg: InstallArg) {
    ic_cdk::println!(
        "[post_upgrade]: upgrade canister executed with install_arg: {:?}",
        install_arg
    );

    match install_arg {
        InstallArg::Reinstall(_) => ic_cdk::trap("InitArg not valid for reinstall"),
        InstallArg::Upgrade(update_arg) => {
            upgrade::post_upgrade(update_arg.clone());
            if let Some(update_arg) = update_arg {
                if update_arg.ecdsa_key_id.is_some() {
                    setup_timers();
                }
            }
        }
    }

    let state = get_state();
    ic_cdk::println!("[post_upgrade]: state = {:?}", state);
}

#[ic_cdk::init]
fn init(install_arg: InstallArg) {
    ic_cdk::println!(
        "[init] initialized canister with install_arg: {:?}",
        install_arg
    );

    match install_arg {
        InstallArg::Reinstall(init_arg) => {
            initialize_state(State::try_from(init_arg).expect("BUG: failed to initialize minter"))
        }
        InstallArg::Upgrade(_) => ic_cdk::trap("UpdateArg not valid for reinstall"),
    }

    let state = STATE.with_borrow(|state| {
        state
            .as_ref()
            .expect("BUG: state is not initialized")
            .clone()
    });

    ic_cdk::println!("[init] new state = {:?}", state);
    setup_timers();
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|s| s.evm_address.clone()).expect("evm address should be initialized")
}

// --------------
// EVM Management
// --------------

#[ic_cdk::update]
async fn clean_old_spent_txs() {
    spent_transactions::discard_old_transactions()
}

#[ic_cdk::update]
pub async fn create_evm_order_with_tx(
    chain_id: u64,
    tx_hash: String,
    user: u64,
    offramper: String,
    providers: HashMap<PaymentProviderType, PaymentProvider>,
    currency: String,
    amount: u128,
    token_address: Option<String>,
) -> Result<u64> {
    guards::only_controller()?;

    let transaction_variant = match token_address {
        Some(_) => TransactionVariant::Token,
        None => TransactionVariant::Native,
    };

    let estimated_gas_lock =
        Ic2P2ramp::get_average_gas_price(chain_id, &TransactionAction::Commit).await?;
    let estimated_gas_withdraw = Ic2P2ramp::get_average_gas_price(
        chain_id,
        &TransactionAction::Release(transaction_variant),
    )
    .await?;
    let evm_input = EvmOrderInput {
        tx_hash: tx_hash.clone(),
        estimated_gas_lock,
        estimated_gas_withdraw,
    };

    let asset = BlockchainAsset::EVM {
        chain_id,
        token_address,
    };
    order_management::validate_deposit_tx(
        &asset,
        Some(DepositInput::Evm(evm_input)),
        offramper.clone(),
        amount,
    )
    .await?;

    let order_id = order_management::create_order(
        &currency,
        user,
        TransactionAddress {
            address_type: AddressType::EVM,
            address: offramper,
        },
        providers,
        asset,
        amount,
        Some(estimated_gas_lock),
        Some(estimated_gas_withdraw),
        None,
    )
    .await?;

    spent_transactions::mark_tx_hash_as_processed(tx_hash);

    Ok(order_id)
}

#[ic_cdk::query]
fn get_order_tx_log(
    order_id: u64,
    user_token: Option<(u64, String)>,
) -> Result<Option<EvmTransactionLog>> {
    if let Some(user_token) = user_token {
        let user = stable::users::get_user(&user_token.0)?;
        user.validate_session(&user_token.1)?;
    } else {
        guards::only_controller()?;
    }
    Ok(heap::logs::get_transaction_log(order_id))
}

#[ic_cdk::query]
pub fn get_pending_txs() -> Vec<EvmTransactionLog> {
    logs::get_pending_transactions()
}

#[ic_cdk::update]
pub async fn resolve_tx_status(order_id: u64, tx_hash: String, chain_id: u64) {
    let status = transaction::check_transaction_status(&tx_hash, chain_id).await;

    match status {
        TransactionStatus::Confirmed(receipt) => {
            ic_cdk::println!("Transaction {} confirmed!", tx_hash);
            logs::update_transaction_log(order_id, TransactionStatus::Confirmed(receipt));
        }
        TransactionStatus::Failed(reason) => {
            ic_cdk::println!("Transaction {} failed: {}", tx_hash, reason);
            logs::update_transaction_log(order_id, TransactionStatus::Failed(reason));
        }
        TransactionStatus::Pending => {
            ic_cdk::println!("Transaction {} is still pending.", tx_hash);
        }
        TransactionStatus::Unresolved(tx_hash, _) => {
            ic_cdk::println!("Transaction {} is still unresolved.", tx_hash);
        }
        _ => (),
    }
}

// ------------------
// Bitcoin Management
// ------------------

#[ic_cdk::update]
pub async fn create_bitcoin_order_with_tx(
    tx_id: String,
    canister_address: String,
    user: u64,
    offramper: String,
    providers: HashMap<PaymentProviderType, PaymentProvider>,
    currency: String,
    amount: u128,
    rune_id: Option<RuneID>,
) -> Result<()> {
    guards::only_controller()?;

    let bitcoin_input = BitcoinOrderInput {
        tx_id,
        canister_address: canister_address.clone(),
    };

    let asset = BlockchainAsset::Bitcoin {
        rune_id: rune_id.clone(),
    };
    let tx_id = order_management::validate_deposit_tx(
        &asset,
        Some(DepositInput::Bitcoin(bitcoin_input.clone())),
        offramper.clone(),
        amount,
    )
    .await?;

    bitcoin_management::spawn_bitcoin_tx_listener(
        tx_id.unwrap(),
        BitcoinTransactionAction::DepositFunds {
            offramper_providers: providers,
            offramper_address: TransactionAddress {
                address_type: AddressType::Bitcoin,
                address: offramper,
            },
            offramper_id: user,
            asset,
            currency,
            amount,
        },
        bitcoin_input.canister_address,
        rune_id,
        0,
    );

    Ok(())
}

// ------------------
// Solana Management
// ------------------
#[ic_cdk::update]
pub async fn create_solana_order_with_tx(
    signature: String,
    user: u64,
    offramper: String,
    providers: HashMap<PaymentProviderType, PaymentProvider>,
    currency: String,
    amount: u128,
    spl_token: Option<String>,
) -> Result<u64> {
    guards::only_controller()?;

    let asset = BlockchainAsset::Solana {
        spl_token: spl_token.clone(),
    };
    order_management::validate_deposit_tx(
        &asset,
        Some(DepositInput::Solana(SolanaOrderInput {
            signature: signature.clone(),
            mint: spl_token.clone(),
        })),
        offramper.clone(),
        amount,
    )
    .await?;

    let order_id = order_management::create_order(
        &currency,
        user,
        TransactionAddress {
            address_type: AddressType::Solana,
            address: offramper.clone(),
        },
        providers,
        asset,
        amount,
        None,
        None,
        None,
    )
    .await?;

    solana_backend_deposit_funds(offramper, amount as u64, spl_token.clone()).await?;

    spent_transactions::mark_tx_hash_as_processed(signature);

    Ok(order_id)
}

// ---------
// Constants
// ---------

#[ic_cdk::query]
fn print_constants() -> String {
    format!(
        "Order's Lock Time = {}s\n\
        User Session's Expiration Time = {}s\n\
        Lock Nonce Timeout Time = {}s\n\
        Exchange Rate Cache Duration = {}s\n\
        Offramper Fiat Fee = {}%\n\
        Onramper Crypto Fee = {}%\n\
        Evm Retry Attempts = {}\n\
        Evm Max Attempts per Retry = {}\n\
        Evm Attempt Interval = {}",
        heap::LOCK_DURATION_TIME_SECONDS,
        CACHE_DURATION,
        nonce::LOCK_NONCE_TIME_SECONDS,
        Session::EXPIRATION_SECS,
        (100. / types::orders::fees::OFFRAMPER_FIAT_FEE_DENOM as f64),
        (100. / types::orders::fees::ADMIN_CRYPTO_FEE_DENOM as f64),
        transaction::MAX_RETRY_ATTEMPTS,
        transaction::MAX_ATTEMPTS_PER_RETRY,
        transaction::ATTEMPT_INTERVAL_SECONDS
    )
}

// ------
// Tokens
// ------

#[ic_cdk::query]
fn get_icp_token_info(ledger_principal: Principal) -> Result<IcpToken> {
    get_icp_token(&ledger_principal)
}

#[ic_cdk::update]
async fn register_icp_tokens(icp_canisters: Vec<String>) -> Result<()> {
    guards::only_controller()?;
    ICPRamp::register_icp_token(icp_canisters).await
}

#[ic_cdk::query]
async fn get_evm_tokens(chain_id: u64) -> Result<Vec<Token>> {
    read_state(|state| {
        let chain_state = state
            .chains
            .get(&chain_id)
            .ok_or(BlockchainError::ChainIdNotFound(chain_id))?;

        Ok(chain_state
            .approved_tokens
            .values()
            .cloned()
            .collect::<Vec<Token>>())
    })
}

#[ic_cdk::update]
async fn register_evm_tokens(chain_id: u64, tokens: Vec<(String, u8, String)>) -> Result<()> {
    guards::only_controller()?;

    let mut new_tokens = TokenManager::new();
    for (token_address, decimals, rate_symbol) in tokens {
        helpers::validate_evm_address(&token_address)?;

        new_tokens.add_token(
            token_address.clone(),
            Token::new(token_address.clone(), decimals, &rate_symbol),
        );
    }

    token::approve_evm_tokens(chain_id, new_tokens.tokens);
    Ok(())
}

// --------
// Balances
// --------

#[ic_cdk::query]
async fn view_canister_balances() -> Result<HashMap<String, f64>> {
    guards::only_controller()?;
    ICPRamp::get_canister_balances().await
}

#[ic_cdk::update]
async fn transfer_canister_funds(
    ledger_canister: Principal,
    to_principal: Principal,
    amount: u128,
) -> Result<()> {
    guards::only_controller()?;

    let fee = get_icp_token(&ledger_canister)?.fee;
    let to_account = Account {
        owner: to_principal,
        subaccount: None,
    };

    ICPRamp::transfer(
        ledger_canister,
        to_account,
        NumTokens::from(amount) - fee.clone(),
        Some(fee),
    )
    .await?;

    Ok(())
}

#[ic_cdk::update]
async fn withdraw_evm_fees(chain_id: u64, amount: u128, token: Option<String>) -> Result<()> {
    guards::only_controller()?;

    let canister_address =
        read_state(|s| s.evm_address.clone()).expect("evm address should be initialized");
    if let Some(token_address) = token.clone() {
        token::evm_token_is_approved(chain_id, &token_address)?;
    }

    Ic2P2ramp::withdraw_deposit(chain_id, 0, canister_address, token, amount, 0).await?;

    Ok(())
}

#[ic_cdk::update]
async fn transfer_evm_funds(
    chain_id: u64,
    to: String,
    amount: u128,
    token: Option<String>,
    estimated_gas: Option<u64>,
) -> Result<()> {
    guards::only_controller()?;
    helpers::validate_evm_address(&to)?;

    if let Some(token) = token.clone() {
        token::evm_token_is_approved(chain_id, &token)?;
    }
    Ic2P2ramp::transfer(chain_id, &to, amount, token, estimated_gas).await
}

// -----
// USERS
// -----

#[ic_cdk::update]
async fn register_user(
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
    login_address: LoginAddress,
    password: Option<String>,
) -> Result<User> {
    user_management::register_user(user_type, payment_providers, login_address, password).await
}

#[ic_cdk::update]
async fn authenticate_user(
    login_address: LoginAddress,
    auth_data: Option<AuthenticationData>,
) -> Result<User> {
    login_address.validate()?;
    let user_id = stable::users::find_user_by_login_address(&login_address)?;
    let user = stable::users::get_user(&user_id)?;
    user.verify_user_auth(auth_data)?;

    user_management::set_session(user_id, &Session::new().await?)
}

#[ic_cdk::update]
async fn update_password(login_address: LoginAddress, new_password: Option<String>) -> Result<()> {
    // the token that is passed in the email,
    // should be generated and stored in the backend canister
    // and passed down to this function
    user_management::reset_password_user(login_address, new_password).await
}

#[ic_cdk::update]
async fn generate_auth_message(login_address: LoginAddress) -> Result<String> {
    login_address.validate()?;

    let address = match login_address.clone() {
        LoginAddress::EVM { address } => Ok(address),
        LoginAddress::Bitcoin { address } => Ok(address),
        LoginAddress::Solana { address } => Ok(address),
        _ => Err(SystemError::InvalidInput(
            "Login address is not of type EVM or Bitcoin".to_string(),
        )),
    }?;

    let user_id = stable::users::find_user_by_login_address(&login_address)?;
    let auth_message = format!(
        "Please sign this message to authenticate: {}\nNonce: {}",
        address,
        random::generate_token().await?
    );

    user_management::update_user_auth_message(user_id, &auth_message)?;

    Ok(auth_message)
}

#[ic_cdk::query]
fn refetch_user(user_id: u64, token: String) -> Result<User> {
    let user = stable::users::get_user(&user_id)?;
    user.validate_session(&token)?;
    Ok(user)
}

#[ic_cdk::query]
fn get_user(user_id: u64) -> Result<User> {
    guards::only_controller()?;
    stable::users::get_user(&user_id)
}

#[ic_cdk::update]
fn remove_user(user_id: u64) -> Result<User> {
    guards::only_controller()?;
    stable::users::remove_user(&user_id)
}

#[ic_cdk::update]
fn add_user_transaction_address(
    user_id: u64,
    token: String,
    address: TransactionAddress,
) -> Result<()> {
    user_management::add_transaction_address(user_id, &token, address)
}

#[ic_cdk::update]
async fn add_user_payment_provider(
    user_id: u64,
    token: String,
    payment_provider: PaymentProvider,
) -> Result<()> {
    user_management::add_payment_provider(user_id, &token, payment_provider).await
}

#[ic_cdk::update]
fn remove_user_payment_provider(
    user_id: u64,
    token: String,
    payment_provider: PaymentProvider,
) -> Result<()> {
    user_management::remove_payment_provider(user_id, &token, &payment_provider)
}

// ------
// Stripe
// ------
#[ic_cdk::update]
async fn stripe_create_express_account(
    email: String,
    country: String,
    platform_label: Option<String>,
) -> Result<String> {
    create_express_account(&email, &country, platform_label).await
}

#[ic_cdk::update]
async fn stripe_get_account_info(
    account_id: String,
    platform_label: Option<String>,
) -> Result<StripeAccountInfo> {
    get_account_info(&account_id, platform_label).await
}

#[ic_cdk::update]
async fn stripe_create_account_link(
    account_id: String,
    refresh_url: String,
    return_url: String,
    platform_label: Option<String>,
) -> Result<String> {
    create_account_link(&account_id, &refresh_url, &return_url, platform_label).await
}

// ------------
// Order Prices
// ------------
#[ic_cdk::update]
async fn get_exchange_rate(fiat_symbol: String, base_asset: RateAsset) -> Result<f64> {
    let quote_asset = RateAsset::Fiat {
        symbol: fiat_symbol,
    };

    rates::get_exchange_rate(base_asset, quote_asset).await
}

// <gas, gas_price>
#[ic_cdk::update]
async fn get_average_gas_prices(
    chain_id: u64,
    max_blocks_in_past: u64,
    method: TransactionAction,
) -> Result<Option<(u64, u128)>> {
    let block = fees::eth_get_latest_block(chain_id, BlockTag::Latest)
        .await
        .map(|block| block.number)?;

    gas::get_average_gas(chain_id, block, Some(max_blocks_in_past), &method)
}

#[ic_cdk::update]
async fn calculate_order_fees(
    asset: BlockchainAsset,
    crypto_amount: u128,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<FeeQuote> {
    let total_fee = order_management::order_crypto_fee(
        asset.clone(),
        crypto_amount,
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await?;

    let admin_fee = get_admin_fee(crypto_amount);
    let blockchain_fee = total_fee.saturating_sub(admin_fee);

    Ok(FeeQuote {
        blockchain_fee,
        admin_fee,
        total_fee,
    })
}

#[ic_cdk::update]
async fn calculate_order_price(
    currency: String,
    asset: BlockchainAsset,
    amount: u128,
) -> Result<(u64, u64)> {
    order_management::calculate_price_and_fee(&currency, &asset, amount).await
}

#[ic_cdk::query]
async fn get_offramper_fee(price: u64) -> u64 {
    price / types::orders::fees::OFFRAMPER_FIAT_FEE_DENOM
}

// ------
// Orders
// ------

#[ic_cdk::query]
fn get_orders(
    filter: Option<OrderFilter>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Vec<OrderState> {
    order_management::get_orders(filter, page, page_size)
}

#[ic_cdk::query]
fn get_order(order_id: u64) -> Result<OrderState> {
    memory::stable::orders::get_order(&order_id)
}

#[ic_cdk::update]
async fn create_order(
    session_token: String,
    currency: String,
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    asset: BlockchainAsset,
    crypto_amount: u128,
    offramper_address: TransactionAddress,
    offramper_user_id: u64,
    deposit_input: Option<DepositInput>,
) -> Result<Option<u64>> {
    let user = stable::users::get_user(&offramper_user_id)?;
    user.validate_session(&session_token)?;
    user.is_banned()?;
    user.is_offramper()?;

    for (provider_type, provider) in &offramper_providers {
        if !user.payment_providers.contains(provider) {
            return Err(UserError::ProviderNotInUser(provider_type.clone()))?;
        }
    }

    let tx_hash = order_management::validate_deposit_tx(
        &asset,
        deposit_input.clone(),
        offramper_address.clone().address,
        crypto_amount,
    )
    .await?;

    match asset.clone() {
        BlockchainAsset::Bitcoin { rune_id } => {
            let canister_address = match deposit_input {
                Some(DepositInput::Bitcoin(v)) => Ok(v.canister_address),
                _ => Err(OrderError::InvalidInput(
                    "Missing bitcoin order input".to_string(),
                )),
            }?;

            bitcoin_management::spawn_bitcoin_tx_listener(
                tx_hash.unwrap(),
                BitcoinTransactionAction::DepositFunds {
                    offramper_providers,
                    offramper_address,
                    offramper_id: offramper_user_id,
                    asset,
                    currency,
                    amount: crypto_amount,
                },
                canister_address,
                rune_id,
                0,
            );

            Ok(None)
        }
        BlockchainAsset::Solana { spl_token } => {
            let order_id = order_management::create_order(
                &currency,
                offramper_user_id,
                offramper_address.clone(),
                offramper_providers,
                asset,
                crypto_amount,
                None,
                None,
                None,
            )
            .await?;

            // Escrow-accounting only (no L1 tx): persist deposit in Solana backend
            solana_backend_deposit_funds(
                offramper_address.address,
                crypto_amount as u64,
                spl_token,
            )
            .await?;

            if let Some(sig) = tx_hash {
                spent_transactions::mark_tx_hash_as_processed(sig);
            }

            Ok(Some(order_id))
        }
        // shared ICP and EVM arm branches, though icp's gas lock and withdraw are None
        _ => {
            let (gas_lock, gas_withdraw) = match deposit_input {
                Some(DepositInput::Evm(v)) => {
                    (Some(v.estimated_gas_lock), Some(v.estimated_gas_withdraw))
                }
                _ => (None, None),
            };

            let order_id = order_management::create_order(
                &currency,
                offramper_user_id,
                offramper_address,
                offramper_providers,
                asset,
                crypto_amount,
                gas_lock,
                gas_withdraw,
                None,
            )
            .await?;

            if let Some(tx_hash) = tx_hash {
                spent_transactions::mark_tx_hash_as_processed(tx_hash);
            };

            Ok(Some(order_id))
        }
    }
}

#[ic_cdk::update]
async fn freeze_order(order_id: u64, user_id: u64, session_token: String) -> Result<()> {
    let order = orders::get_order(&order_id)?.created()?;
    let user = memory::stable::users::get_user(&user_id)?;
    user.validate_session(&session_token)?;
    if order.offramper_user_id != user_id {
        return Err(UserError::Unauthorized.into());
    }
    orders::set_processing_order(&order_id)
}

#[ic_cdk::update]
async fn unfreeze_order(order_id: u64) -> Result<()> {
    guards::only_controller()?;
    orders::unset_processing_order(&order_id)
}

#[ic_cdk::update]
async fn top_up_order(
    order_id: u64,
    user_id: u64,
    session_token: String,
    amount: u128,
    deposit_input: Option<DepositInput>,
) -> Result<()> {
    let user = memory::stable::users::get_user(&user_id)?;
    user.validate_session(&session_token)?;

    let order = orders::get_order(&order_id)?.created()?;
    if order.offramper_user_id != user_id {
        return Err(UserError::Unauthorized.into());
    }
    orders::set_processing_order(&order_id)?;

    let tx_hash = order_management::validate_deposit_tx(
        &order.crypto.asset,
        deposit_input.clone(),
        order.offramper_address.clone().address,
        amount,
    )
    .await
    .inspect_err(|_e| {
        let _ = orders::unset_processing_order(&order_id);
    })?;

    match order.crypto.asset.clone() {
        BlockchainAsset::Bitcoin { rune_id } => {
            let canister_address = match deposit_input {
                Some(DepositInput::Bitcoin(v)) => Ok(v.canister_address),
                _ => Err(OrderError::InvalidInput(
                    "Missing bitcoin topup input".into(),
                )),
            }?;

            bitcoin_management::spawn_bitcoin_tx_listener(
                tx_hash.clone().ok_or(OrderError::InvalidInput(
                    "Missing bitcoin tx id".to_string(),
                ))?,
                BitcoinTransactionAction::TopUpFunds {
                    order_id,
                    offramper_address: order.offramper_address.address.clone(),
                    amount,
                },
                canister_address,
                rune_id,
                0,
            );

            return orders::unset_processing_order(&order_id);
        }

        BlockchainAsset::Solana { spl_token } => {
            // 1) Reflect escrow-accounting deposit first (like create_order)
            solana_backend_deposit_funds(
                order.offramper_address.address.clone(),
                amount as u64,
                spl_token,
            )
            .await?;

            if let Some(sig) = tx_hash {
                spent_transactions::mark_tx_hash_as_processed(sig);
            }

            // 2) Bump order amount + fee (no L1 wait needed here)
            order_management::topup_order(&order, amount, None, None)
                .await
                .inspect_err(|_| {
                    let _ = orders::unset_processing_order(&order_id);
                })?;

            return orders::unset_processing_order(&order_id);
        }

        // EVM / ICP
        _ => {
            if let Some(tx) = tx_hash {
                spent_transactions::mark_tx_hash_as_processed(tx);
            }

            let (gas_lock, gas_withdraw) = match deposit_input {
                Some(DepositInput::Evm(v)) => {
                    (Some(v.estimated_gas_lock), Some(v.estimated_gas_withdraw))
                }
                _ => (None, None),
            };
            order_management::topup_order(&order, amount, gas_lock, gas_withdraw)
                .await
                .inspect_err(|_| {
                    let _ = orders::unset_processing_order(&order_id);
                })?;

            return orders::unset_processing_order(&order_id);
        }
    }
}

#[ic_cdk::update]
async fn lock_order(
    order_id: u64,
    session_token: String,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    lock_amount: u128,
    stripe_success_url: Option<String>,
    stripe_cancel_url: Option<String>,
) -> Result<()> {
    orders::set_processing_order(&order_id)?;

    ic_cdk::println!("[lock_order]");
    if let Err(e) = order_management::lock_order(
        order_id,
        session_token,
        onramper_user_id,
        onramper_provider,
        onramper_address,
        lock_amount,
        stripe_success_url,
        stripe_cancel_url,
    )
    .await
    {
        orders::unset_processing_order(&order_id)?;
        return Err(e);
    };

    Ok(())
}

#[ic_cdk::update]
async fn retry_order_unlock(order_id: u64) -> Result<()> {
    guards::only_controller()?;
    orders::set_processing_order(&order_id)?;

    if let Err(e) = management::order::unlock_order(order_id).await {
        orders::unset_processing_order(&order_id)?;
        return Err(e);
    };

    Ok(())
}

#[ic_cdk::update]
async fn cancel_order(order_id: u64, session_token: String) -> Result<()> {
    orders::set_processing_order(&order_id)?;

    if let Err(e) = order_management::cancel_order(order_id, session_token).await {
        orders::unset_processing_order(&order_id)?;
        return Err(e);
    };

    Ok(())
}

// ---------------
// Revolut Payment
// ---------------
#[ic_cdk::query]
async fn execute_revolut_payment(order_id: u64, session_token: String) -> Result<String> {
    revolut_token::wait_for_revolut_access_token(order_id, &session_token, 10, 3).await
}

// --------------------
// Payment Verification
// --------------------
#[ic_cdk::query]
fn verify_order_is_payable(order_id: u64, session_token: String) -> Result<()> {
    let _ = order_management::verify_order_is_payable(order_id, Some(session_token))?;
    Ok(())
}

#[ic_cdk::update]
async fn retry_order_completion(order_id: u64) -> Result<()> {
    guards::only_controller()?;

    let order = memory::stable::orders::get_order(&order_id)?.locked()?;
    if !order.payment_done {
        return Err(OrderError::PaymentVerificationFailed)?;
    };

    payment_management::handle_payment_completion(&order).await
}

#[ic_cdk::update]
async fn verify_transaction(
    order_id: u64,
    session_token: Option<String>,
    transaction_id: String,
) -> Result<()> {
    ic_cdk::println!(
        "[verify_transaction] Starting verification for order ID: {} and transaction ID: {}",
        order_id,
        transaction_id
    );

    orders::set_processing_order(&order_id)?;

    if let Err(e) = process_transaction(order_id, session_token, transaction_id).await {
        orders::unset_processing_order(&order_id)?;
        return Err(e);
    }

    Ok(())
}

async fn process_transaction(
    order_id: u64,
    session_token: Option<String>,
    transaction_id: String,
) -> Result<()> {
    let order = order_management::verify_order_is_payable(order_id, session_token)?;

    match &order.clone().onramper.provider {
        PaymentProvider::PayPal { id: onramper_id } => {
            ic_cdk::println!("[verify_transaction] Handling Paypal payment verification");

            payment_management::paypal::verify_paypal_payment(onramper_id, &transaction_id, &order)
                .await?
        }

        PaymentProvider::Revolut {
            scheme: onramper_scheme,
            id: onramper_id,
            name: _,
        } => {
            ic_cdk::println!("[verify_transaction] Handling Revolut payment verification");

            payment_management::revolut::verify_revolut_payment(
                onramper_id,
                onramper_scheme,
                &transaction_id,
                &order,
            )
            .await?
        }

        PaymentProvider::Stripe { .. } => return Err(OrderError::InvalidOnramperProvider.into()),

        PaymentProvider::Email { email } => {
            ic_cdk::println!("[verify_transaction] Handling Stripe checkout verification");

            payment_management::stripe::verify_stripe_payment(&order, email).await?;
        }
    }

    payment_management::handle_payment_completion(&order).await
}

ic_cdk::export_candid!();
