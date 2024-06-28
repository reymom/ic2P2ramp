mod errors;
mod evm;
mod management;
mod outcalls;
mod state;

use errors::{RampError, Result};
use std::time::Duration;

use evm::transaction::spawn_transaction_checker;
use evm::{providers, rpc::ProviderView, vault::Ic2P2ramp};
use management::order as order_management;
use management::user as user_management;
use outcalls::{paypal_auth, paypal_capture, xrc_rates};
use state::storage::{OrderState, PaymentProvider, User};
use state::{initialize_state, mutate_state, read_state, InitArg};

pub const SCRAPING_LOGS_INTERVAL: Duration = Duration::from_secs(3 * 60);

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

#[ic_cdk::init]
fn init(arg: InitArg) {
    ic_cdk::println!("[init]: initialized minter with arg: {:?}", arg);
    initialize_state(state::State::try_from(arg).expect("BUG: failed to initialize minter"));
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
async fn test_deposit_funds(
    chain_id: u64,
    amount: u64,
    token_address: Option<String>,
    gas: Option<i32>,
) -> Result<String> {
    Ic2P2ramp::deposit_funds(chain_id, amount, token_address, gas).await
}

// ----------
// Management
// ----------

#[ic_cdk::update]
async fn approve_token_allowance(chain_id: u64, token_address: String, gas: i32) -> Result<()> {
    Ic2P2ramp::approve_token_allowance(chain_id, token_address, gas).await
}

#[ic_cdk::query]
async fn get_rpc_providers() -> Vec<ProviderView> {
    providers::get_providers().await
}

// ---------
// XRC Rate
// ---------

#[ic_cdk::update]
async fn get_usd_exchange_rate(fiat_symbol: String, crypto_symbol: String) -> Result<String> {
    match xrc_rates::get_exchange_rate(&fiat_symbol, &crypto_symbol).await {
        Ok(rate) => Ok(rate.to_string()),
        Err(err) => Err(err),
    }
}

// -----
// USERS
// -----

#[ic_cdk::update]
fn register_user(evm_address: String, payment_providers: Vec<PaymentProvider>) -> Result<String> {
    user_management::register_user(evm_address, payment_providers)
}

#[ic_cdk::query]
fn get_user(evm_address: String) -> Result<User> {
    user_management::get_user(evm_address)
}

#[ic_cdk::query]
fn add_payment_provider(evm_address: String, payment_provider: PaymentProvider) -> Result<()> {
    user_management::add_payment_provider(evm_address, payment_provider)
}

// ------------------
// ICP Offramp Orders
// ------------------

#[ic_cdk::query]
fn get_orders() -> Vec<OrderState> {
    order_management::get_orders()
}

#[ic_cdk::update]
fn create_order(
    fiat_amount: u64,
    fiat_symbol: String,
    crypto_amount: u64,
    offramper_providers: Vec<PaymentProvider>,
    address: String,
    chain_id: u64,
    token_address: Option<String>,
) -> Option<String> {
    order_management::create_order(
        fiat_amount,
        fiat_symbol,
        crypto_amount,
        offramper_providers,
        address,
        chain_id,
        token_address,
    )
}

#[ic_cdk::update]
async fn lock_order(
    order_id: String,
    onramper_provider: PaymentProvider,
    onramper_address: String,
    gas: Option<i32>,
) -> Result<()> {
    if !user_management::can_commit_order(&onramper_address)? {
        return Err(RampError::UserBanned);
    }

    let order_state = order_management::get_order_state_by_id(order_id.as_str())?;
    let order = match order_state {
        OrderState::Created(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let tx_hash = Ic2P2ramp::commit_deposit(
        order.chain_id,
        order.offramper_address,
        order.token_address,
        order.crypto_amount,
        gas,
    )
    .await?;
    spawn_transaction_checker(
        tx_hash,
        order.chain_id,
        60,
        Duration::from_secs(4),
        move || {
            let _ = order_management::lock_order(
                order_id.as_str(),
                onramper_provider.clone(),
                onramper_address.clone(),
            );
        },
    );
    Ok(())
}

#[ic_cdk::update]
async fn unlock_order(order_id: String, gas: Option<u64>) -> Result<()> {
    let order_state = order_management::get_order_state_by_id(order_id.as_str())?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let tx_hash = Ic2P2ramp::uncommit_deposit(
        order.base.chain_id,
        order.base.offramper_address,
        order.base.token_address,
        order.base.crypto_amount,
        gas,
    )
    .await?;
    spawn_transaction_checker(
        tx_hash,
        order.base.chain_id,
        60,
        Duration::from_secs(4),
        move || {
            let _ = order_management::unlock_order(order_id.as_str());
        },
    );
    Ok(())
}

#[ic_cdk::update]
fn cancel_order(order_id: String) -> Result<String> {
    // let order = management::get_order_by_id(order_id.clone()).await?;

    // evm::vault::withdraw(order.chain_id, order.crypto_amount).await?;

    order_management::cancel_order(order_id.as_str())
}

// ---------------
// Paypal Payment
// ---------------

#[ic_cdk::update]
async fn verify_transaction(
    order_id: String,
    transaction_id: String,
    gas: Option<i32>,
) -> Result<()> {
    let order_state = order_management::get_order_state_by_id(order_id.as_str())?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let cycles: u128 = 10_000_000_000;
    let access_token = paypal_auth::get_paypal_access_token().await?;
    let capture_details =
        paypal_capture::fetch_paypal_capture_details(&access_token, &transaction_id, cycles)
            .await?;

    // Verify the captured payment details
    let expected_fiat_amount = order.base.fiat_amount as f64 / 100.0; // fiat_amount is in cents
    let received_amount: f64 = capture_details
        .amount
        .value
        .parse()
        .map_err(RampError::from)?;
    ic_cdk::println!("received_amount = {}", received_amount);

    let amount_matches = (received_amount - expected_fiat_amount).abs() < f64::EPSILON;
    let currency_matches = capture_details.amount.currency_code == order.base.currency_symbol;
    let offramper_matches =
        capture_details.payee.email_address == order.base.offramper_providers[0].get_id();
    // let onramper_matches = order.onramper_paypal_id.as_deref()
    //     == Some(&capture_details.supplementary_data.related_ids.order_id);

    if capture_details.status == "COMPLETED"
        && amount_matches
        && currency_matches
        && offramper_matches
    // && onramper_matches
    {
        order_management::mark_order_as_paid(order.base.id.as_str())?;
        let tx_hash = Ic2P2ramp::release_funds(order_id.as_str(), gas).await?;
        spawn_transaction_checker(
            tx_hash,
            order.base.chain_id,
            60,
            Duration::from_secs(4),
            move || {
                // Update order state to completed
                let _ = management::order::update_order_state(
                    order_id.as_str(),
                    OrderState::Completed(order_id.clone()),
                );
            },
        );
        Ok(())
    } else {
        Err(RampError::PaymentVerificationFailed)
    }
}

ic_cdk::export_candid!();
