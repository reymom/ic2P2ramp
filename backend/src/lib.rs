mod evm;
mod icp;
mod management;
mod model;
mod outcalls;

use candid::Principal;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};
use num_traits::cast::ToPrimitive;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use evm::{
    fees,
    rpc::{Block, BlockTag, TransactionReceipt},
    transaction,
    vault::Ic2P2ramp,
};
use icp::vault::Ic2P2ramp as ICPRamp;
use management::{
    order as order_management, payment as payment_management, random, user as user_management,
};
use model::errors::{self, RampError, Result};
use model::memory::{
    heap::{self, initialize_state, mutate_state, read_state, upgrade, InitArg, State},
    stable,
};
use model::types::evm::{
    chains,
    gas::{self, MethodGasUsage},
    logs::{EvmTransactionLog, TransactionAction},
    token::{self, Token, TokenManager},
};
use model::types::{
    self, get_icp_fee, is_icp_token_supported,
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

fn setup_timers() {
    ic_cdk_timers::set_timer(Duration::ZERO, || {
        ic_cdk::spawn(async {
            let public_key = evm::signer::get_public_key().await;
            let evm_address = evm::signer::pubkey_bytes_to_address(&public_key);
            mutate_state(|s| {
                s.ecdsa_pub_key = Some(public_key);
                s.evm_address = Some(evm_address);
            });
        })
    });
}

#[ic_cdk::pre_upgrade]
fn pre_upgrade() {
    upgrade::pre_upgrade()
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
    upgrade::post_upgrade()
}

#[ic_cdk::init]
fn init(arg: InitArg) {
    ic_cdk::println!("[init]: initialized minter with arg: {:?}", arg);
    initialize_state(State::try_from(arg).expect("BUG: failed to initialize minter"));
    setup_timers();
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|s| s.evm_address.clone()).expect("evm address should be initialized")
}

#[ic_cdk::update]
async fn transfer_eth(
    chain_id: u64,
    to: String,
    amount: u128,
    estimated_gas: Option<u64>,
) -> Result<String> {
    guards::only_controller()?;
    helpers::validate_evm_address(&to)?;
    Ic2P2ramp::transfer(chain_id, &to, amount, None, estimated_gas).await
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

// ----------
// Management
// ----------

#[ic_cdk::update]
async fn register_icp_tokens(icp_canisters: Vec<String>) -> Result<()> {
    guards::only_controller()?;
    ICPRamp::set_icp_fees(icp_canisters).await
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
async fn register_evm_tokens(
    chain_id: u64,
    tokens: Vec<(String, u8, String, Option<String>)>,
) -> Result<()> {
    guards::only_controller()?;

    let mut new_tokens = TokenManager::new();
    for (token_address, decimals, rate_symbol, desc) in tokens {
        helpers::validate_evm_address(&token_address)?;

        let mut new_token = Token::new(token_address.clone(), decimals, &rate_symbol);
        new_token.set_description(desc);

        new_tokens.add_token(token_address, new_token)
    }

    token::approve_evm_tokens(chain_id, new_tokens.tokens);
    Ok(())
}

#[ic_cdk::query]
fn get_icp_transaction_fee(ledger_principal: Principal) -> Result<candid::Nat> {
    get_icp_fee(&ledger_principal)
}

#[ic_cdk::query]
async fn view_canister_balances() -> Result<HashMap<String, f64>> {
    ICPRamp::get_canister_balances().await
}

#[ic_cdk::update]
async fn transfer_canister_funds(
    ledger_canister: Principal,
    to_principal: Principal,
    amount: u128,
) -> Result<()> {
    guards::only_controller()?;

    let fee = get_icp_fee(&ledger_canister)?;
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
    if let Some(token_address) = token {
        token::evm_token_is_approved(chain_id, &token_address)?;
        let withdraw_tx = Ic2P2ramp::withdraw_token(chain_id, token_address, amount).await?;
        Ok(withdraw_tx)
    } else {
        let withdraw_tx = Ic2P2ramp::withdraw_base_currency(chain_id, amount).await?;
        Ok(withdraw_tx)
    }
}

// ---------
// XRC Rate
// ---------

#[ic_cdk::update]
async fn get_exchange_rate(fiat_symbol: String, crypto_symbol: String) -> Result<String> {
    let base_asset = Asset {
        class: AssetClass::Cryptocurrency,
        symbol: crypto_symbol.to_string(),
    };
    let quote_asset = Asset {
        class: AssetClass::FiatCurrency,
        symbol: fiat_symbol.to_string(),
    };

    match xrc_rates::get_exchange_rate(base_asset, quote_asset).await {
        Ok(rate) => Ok(rate.to_string()),
        Err(err) => Err(err),
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
    fiat_amount: u64,
    crypto_amount: u128,
    token: Option<String>,
    estimated_gas_lock: u64,
    estimated_gas_withdraw: u64,
) -> Result<(u64, u128)> {
    order_management::calculate_order_evm_fees(
        chain_id,
        fiat_amount,
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
    fiat_amount: u64,
    fiat_symbol: String,
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
        offramper_user_id,
        fiat_amount,
        fiat_symbol,
        offramper_providers,
        blockchain,
        token_address,
        crypto_amount,
        offramper_address,
        estimated_gas_lock,
        estimated_gas_withdraw,
    )
    .await
}

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

    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Created(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    if !types::contains_provider_type(&onramper_provider, &order.offramper_providers) {
        return Err(RampError::InvalidOnramperProvider);
    }

    let (revolut_consent_id, consent_url) =
        payment_management::get_revolut_consent(&order, &onramper_provider).await?;

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
            transaction::spawn_transaction_checker(
                tx_hash.clone(),
                chain_id,
                60,
                Duration::from_secs(4),
                order_id,
                TransactionAction::Commit,
                move |receipt: TransactionReceipt| {
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

                    let onramper_provider = onramper_provider.clone();
                    let onramper_address = onramper_address.clone();
                    let revolut_consent_id = revolut_consent_id.clone();
                    let consent_url = consent_url.clone();

                    ic_cdk::spawn(async move {
                        match order_management::lock_order(
                            order_id,
                            onramper_user_id,
                            onramper_provider,
                            onramper_address,
                            revolut_consent_id,
                            consent_url,
                        )
                        .await
                        {
                            Ok(()) => ic_cdk::println!("order {:?} is locked!", order_id),
                            Err(err) => {
                                ic_cdk::println!(
                                    "order {:?} failed to be locked: {:?}",
                                    order_id,
                                    err
                                )
                            }
                        };
                    });
                },
            );
            Ok(tx_hash)
        }
        Blockchain::ICP { .. } => {
            order_management::lock_order(
                order_id,
                onramper_user_id,
                onramper_provider.clone(),
                onramper_address.clone(),
                revolut_consent_id.clone(),
                consent_url.clone(),
            )
            .await?;
            Ok(format!("order {:?} is locked!", order_id))
        }
        _ => return Err(RampError::UnsupportedBlockchain),
    }
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

    let user = stable::users::get_user(&order.onramper_user_id)?;
    user.validate_session(&session_token)?;
    user.validate_onramper()?;

    order_management::unlock_order(order_id, estimated_gas).await
}

#[ic_cdk::update]
async fn manual_unlock_order(order_id: u64) -> Result<()> {
    guards::only_controller()?;

    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    if order.uncommited {
        stable::orders::unlock_order(order_id)?;
        ic_cdk::println!("Manually unlocked order #{}", order_id);
        Ok(())
    } else {
        return Err(RampError::InternalError(
            "order is not uncommited".to_string(),
        ));
    }
}

#[ic_cdk::update]
async fn cancel_order(order_id: u64, session_token: String) -> Result<()> {
    let order_state = stable::orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Created(order) => order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let user = stable::users::get_user(&order.offramper_user_id)?;
    user.is_offramper()?;
    user.validate_session(&session_token)?;

    match &order.crypto.blockchain {
        Blockchain::ICP { ledger_principal } => {
            let offramper_principal =
                Principal::from_text(&order.offramper_address.address).unwrap();

            let amount = NumTokens::from(order.crypto.amount);
            let fee = get_icp_fee(ledger_principal)?;

            let to_account = Account {
                owner: offramper_principal,
                subaccount: None,
            };
            ic_cdk::println!("[cancel] amount = {:?}", amount);
            ic_cdk::println!("[cancel] fee = {:?}", fee);
            ICPRamp::transfer(
                *ledger_principal,
                to_account,
                amount - fee.clone(),
                Some(fee),
            )
            .await?;
        }
        _ => (),
    }

    order_management::cancel_order(order_id)
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

    match &order.clone().onramper_provider {
        PaymentProvider::PayPal { id: onramper_id } => {
            ic_cdk::println!("[verify_transaction] Handling Paypal payment verification");

            payment_management::verify_paypal_payment(onramper_id, &transaction_id, &order.base)
                .await?
        }

        PaymentProvider::Revolut {
            scheme: onramper_scheme,
            id: onramper_id,
            name: _,
        } => {
            ic_cdk::println!("[verify_transaction] Handling Revolut payment verification");

            payment_management::verify_revolut_payment(
                onramper_id,
                &transaction_id,
                onramper_scheme,
                &order.base,
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
