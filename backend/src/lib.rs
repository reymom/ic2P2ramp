mod evm;
mod icp;
mod management;
mod model;
mod outcalls;

use std::collections::{HashMap, HashSet};

use candid::Principal;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use evm::{
    fees,
    rpc::{Block, BlockTag},
    transaction::{self, TransactionStatus},
    vault::Ic2P2ramp,
};
use icp::vault::Ic2P2ramp as ICPRamp;
use management::{
    order as order_management, payment as payment_management, random, user as user_management,
};
use model::errors::{self, RampError, Result};
use model::memory::{
    heap::{self, initialize_state, read_state, setup_timers, upgrade, InstallArg, State, STATE},
    stable,
};
use model::types::evm::{
    chains,
    gas::{self, MethodGasUsage},
    logs::EvmTransactionLog,
    token::{self, Token, TokenManager},
};
use model::types::{
    self,
    exchange_rate::CACHE_DURATION,
    icp::{get_icp_token, is_icp_token_supported, IcpToken},
    order::{OrderFilter, OrderState},
    session::Session,
    user::{User, UserType},
    AuthenticationData, Blockchain, LoginAddress, PaymentProvider, PaymentProviderType,
    TransactionAddress,
};
use model::{guards, helpers};
use outcalls::{
    paypal,
    revolut::{self, token as revolut_token},
    xrc_rates::{self, Asset, AssetClass},
};

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    upgrade::pre_upgrade()
}

#[ic_cdk::post_upgrade]
fn post_upgrade(install_arg: InstallArg) {
    ic_cdk::println!(
        "[post_upgrade]: upgrad canister executed with install_arg: {:?}",
        install_arg
    );

    match install_arg {
        InstallArg::Reinstall(_) => ic_cdk::trap("InitArg not valid for reinstall"),
        InstallArg::Upgrade(update_arg) => {
            upgrade::post_upgrade(update_arg.clone());
            if let Some(update_arg) = update_arg {
                ic_cdk::println!("[post_upgrade] some update_arg!");
                if update_arg.ecdsa_key_id.is_some() {
                    ic_cdk::println!("[post_upgrade] some ecdsa_key_id!");
                    setup_timers();
                }
            }
        }
    }

    let state = STATE.with_borrow(|state| {
        state
            .as_ref()
            .expect("BUG: state is not initialized")
            .clone()
    });
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

// -----
// Tests
// -----

#[ic_cdk::update]
async fn test_paypal() -> Result<String> {
    paypal::auth::get_paypal_access_token().await
}

#[ic_cdk::update]
async fn test_get_consent_url() -> Result<String> {
    let consent_id = revolut::consent::create_account_access_consent(
        "1.00",
        "GBP",
        "UK.OBIE.IBAN",
        "GB14REVO04290956685580",
        "UK.OBIE.SortCodeAccountNumber",
        "04290956685580",
        "Jan Smith",
    )
    .await?;

    Ok(revolut::authorize::get_authorization_url(&consent_id).await?)
}

#[ic_cdk::update]
async fn test_get_revolut_payment_token(consent_id: String) -> Result<String> {
    revolut::token::get_revolut_access_token(consent_id).await
}

#[ic_cdk::update]
async fn test_get_revolut_payment_details(payment_id: String) -> Result<()> {
    let details = revolut::transaction::fetch_revolut_payment_details(&payment_id).await?;
    ic_cdk::println!("details = {:?}", details);
    Ok(())
}

#[ic_cdk::update]
async fn test_get_latest_block(chain_id: u64) -> Result<Block> {
    fees::eth_get_latest_block(chain_id, evm::rpc::BlockTag::Latest).await
}

#[ic_cdk::update]
async fn test_get_transaction_status(tx_hash: String, chain_id: u64) -> TransactionStatus {
    transaction::check_transaction_status(tx_hash, chain_id).await
}

#[ic_cdk::update]
async fn test_get_transaction_count(chain_id: u64) -> Result<u128> {
    transaction::eth_get_transaction_count(chain_id).await
}

#[ic_cdk::update]
async fn test_get_exchange_rate(fiat_symbol: String, crypto_symbol: String) -> Result<f64> {
    guards::only_controller()?;

    let base_asset = Asset {
        class: AssetClass::Cryptocurrency,
        symbol: crypto_symbol.to_string(),
    };
    let quote_asset = Asset {
        class: AssetClass::FiatCurrency,
        symbol: fiat_symbol.to_string(),
    };

    xrc_rates::get_cached_exchange_rate(base_asset, quote_asset).await
}

// ----------
// Management
// ----------

#[ic_cdk::query]
fn print_constants() -> String {
    format!(
        "Order's Lock Time = {}s\n\
        User Session's Expiration Time = {}s\n\
        Exchange Rate Cache Duration = {}s\n\
        Offramper Fiat Fee = {}%\n\
        Onramper Crypto Fee = {}%\n\
        Default Evm Gas = {}\n\
        Evm Retry Attempts = {}\n\
        Evm Max Attempts per Retry = {}\n\
        Evm Attempt Interval = {}",
        heap::LOCK_DURATION_TIME_SECONDS,
        CACHE_DURATION,
        Session::EXPIRATION_SECS,
        (100. / types::order::OFFRAMPER_FIAT_FEE_DENOM as f64),
        (100. / types::order::ADMIN_CRYPTO_FEE_DENOM as f64),
        Ic2P2ramp::DEFAULT_GAS,
        transaction::MAX_RETRY_ATTEMPTS,
        transaction::MAX_ATTEMPTS_PER_RETRY,
        transaction::ATTEMPT_INTERVAL_SECONDS
    )
}

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
            .ok_or_else(|| RampError::ChainIdNotFound(chain_id))?;

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
async fn transfer_evm_funds(
    chain_id: u64,
    to: String,
    amount: u128,
    token: Option<String>,
    estimated_gas: Option<u64>,
) -> Result<String> {
    guards::only_controller()?;
    helpers::validate_evm_address(&to)?;

    if let Some(token) = token.clone() {
        token::evm_token_is_approved(chain_id, &token)?;
    }
    Ic2P2ramp::transfer(chain_id, &to, amount, token, estimated_gas).await
}

#[ic_cdk::update]
async fn withdraw_evm_fees(chain_id: u64, amount: u128, token: Option<String>) -> Result<String> {
    guards::only_controller()?;

    let canister_address =
        read_state(|s| s.evm_address.clone()).expect("evm address should be initialized");
    if let Some(token_address) = token {
        token::evm_token_is_approved(chain_id, &token_address)?;
        let (withdraw_tx, _) =
            Ic2P2ramp::withdraw_token(chain_id, canister_address, token_address, amount, 0).await?;
        Ok(withdraw_tx)
    } else {
        let (withdraw_tx, _) =
            Ic2P2ramp::withdraw_base_currency(chain_id, canister_address, amount, 0).await?;
        Ok(withdraw_tx)
    }
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
async fn generate_evm_auth_message(login_address: LoginAddress) -> Result<String> {
    login_address.validate()?;
    let address = if let LoginAddress::EVM { address } = login_address.clone() {
        Ok(address)
    } else {
        Err(RampError::InvalidInput(
            "Login address is not of type EVM".to_string(),
        ))
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
fn add_user_payment_provider(
    user_id: u64,
    token: String,
    payment_provider: PaymentProvider,
) -> Result<()> {
    user_management::add_payment_provider(user_id, &token, payment_provider)
}

// ------------------
// ICP Offramp Orders
// ------------------

// <gas, gas_price>
#[ic_cdk::update]
async fn get_average_gas_prices(
    chain_id: u64,
    max_blocks_in_past: u64,
    method: MethodGasUsage,
) -> Result<Option<(u128, u128)>> {
    let block = fees::eth_get_latest_block(chain_id, BlockTag::Latest).await?;
    gas::get_average_gas(chain_id, block.number, max_blocks_in_past, &method)
}

// <(offramper_fee, crypto_fee)>
#[ic_cdk::update]
async fn calculate_order_evm_fees(
    chain_id: u64,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_lock: u64,
    estimated_gas_withdraw: u64,
) -> Result<u128> {
    order_management::calculate_order_evm_fees(
        chain_id,
        crypto_amount,
        token.clone(),
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await
}

#[ic_cdk::query]
fn get_orders(
    filter: Option<OrderFilter>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Vec<OrderState> {
    order_management::get_orders(filter, page, page_size)
}

#[ic_cdk::query]
fn get_transaction_log(
    order_id: u64,
    user_id: u64,
    session_token: String,
) -> Result<Option<EvmTransactionLog>> {
    let user = stable::users::get_user(&user_id)?;
    user.validate_session(&session_token)?;
    Ok(heap::logs::get_transaction_log(order_id))
}

#[ic_cdk::update]
async fn create_order(
    session_token: String,
    currency: String,
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
    blockchain: Blockchain,
    token_address: Option<String>,
    crypto_amount: u128,
    offramper_address: TransactionAddress,
    offramper_user_id: u64,
    estimated_gas_lock: Option<u64>,
    estimated_gas_withdraw: Option<u64>,
) -> Result<u64> {
    let user = stable::users::get_user(&offramper_user_id)?;
    user.validate_session(&session_token)?;
    user.is_banned()?;
    user.is_offramper()?;

    for (provider_type, provider) in &offramper_providers {
        if !user.payment_providers.contains(&provider) {
            return Err(RampError::ProviderNotInUser(provider_type.clone()));
        }
    }

    match blockchain {
        Blockchain::EVM { chain_id } => {
            chains::chain_is_supported(chain_id)?;
            if let Some(token) = token_address.clone() {
                token::evm_token_is_approved(chain_id, &token)?;
            };
        }
        Blockchain::ICP { ledger_principal } => is_icp_token_supported(&ledger_principal)?,
        _ => return Err(RampError::UnsupportedBlockchain),
    }

    order_management::create_order(
        &currency,
        offramper_user_id,
        offramper_address,
        offramper_providers,
        blockchain,
        token_address,
        crypto_amount,
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await
}

#[ic_cdk::update]
async fn calculate_order_price(
    currency: String,
    crypto_amount: u128,
    crypto_symbol: String,
) -> Result<(u64, u64)> {
    order_management::calculate_price_and_fee(&currency, crypto_amount, &crypto_symbol).await
}

#[ic_cdk::query]
async fn get_offramper_fee(price: u64) -> u64 {
    price / types::order::OFFRAMPER_FIAT_FEE_DENOM
}

// async fn top_up_order() -> Result<()>

#[ic_cdk::update]
async fn lock_order(
    order_id: u64,
    session_token: String,
    onramper_user_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: TransactionAddress,
    estimated_gas: Option<u64>,
) -> Result<String> {
    let user = stable::users::get_user(&onramper_user_id)?;
    user.validate_session(&session_token)?;
    user.validate_onramper()?;
    user.is_banned()?;

    order_management::lock_order(
        order_id,
        onramper_user_id,
        onramper_provider,
        onramper_address,
        estimated_gas,
    )
    .await
}

#[ic_cdk::update]
async fn unlock_order(
    order_id: u64,
    session_token: String,
    estimated_gas: Option<u64>,
) -> Result<String> {
    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let user = stable::users::get_user(&order.onramper.user_id)?;
    user.validate_session(&session_token)?;
    user.validate_onramper()?;

    order_management::unlock_order(order_id, Some(session_token), estimated_gas).await
}

#[ic_cdk::update]
async fn cancel_order(order_id: u64, session_token: String) -> Result<String> {
    order_management::cancel_order(order_id, session_token).await
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
    let _ = order_management::verify_order_is_payable(order_id, &session_token)?;
    Ok(())
}

#[ic_cdk::update]
async fn verify_transaction(
    order_id: u64,
    session_token: String,
    transaction_id: String,
    estimated_gas: Option<u64>,
) -> Result<String> {
    ic_cdk::println!(
        "[verify_transaction] Starting verification for order ID: {} and transaction ID: {}",
        order_id,
        transaction_id
    );

    let order = order_management::verify_order_is_payable(order_id, &session_token)?;

    match &order.clone().onramper.provider {
        PaymentProvider::PayPal { id: onramper_id } => {
            ic_cdk::println!("[verify_transaction] Handling Paypal payment verification");

            payment_management::verify_paypal_payment(&onramper_id, &transaction_id, &order).await?
        }

        PaymentProvider::Revolut {
            scheme: onramper_scheme,
            id: onramper_id,
            name: _,
        } => {
            ic_cdk::println!("[verify_transaction] Handling Revolut payment verification");

            payment_management::verify_revolut_payment(
                &onramper_id,
                &transaction_id,
                &onramper_scheme,
                &order,
            )
            .await?
        }
    }

    match order.base.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let tx_hash = payment_management::handle_evm_payment_completion(
                order_id,
                chain_id,
                estimated_gas,
            )
            .await?;
            return Ok(tx_hash);
        }
        Blockchain::ICP { ledger_principal } => {
            let index =
                payment_management::handle_icp_payment_completion(order_id, &ledger_principal)
                    .await?;
            return Ok(index);
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
}

ic_cdk::export_candid!();
