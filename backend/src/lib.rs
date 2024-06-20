mod evm;
mod management;
mod outcalls;
mod state;

use std::time::Duration;

use ethers_core::types::U256;
use evm::fees;
use evm::vault::Ic2P2ramp;
use management::order as order_management;
use management::user as user_management;
use outcalls::{paypal_auth, paypal_capture, xrc_rates};
use state::storage::Order;
use state::storage::{PaymentProvider, User};
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

#[ic_cdk::update]
async fn test_deposit_funds(
    chain_id: u64,
    amount: u64,
    token_address: Option<String>,
    gas: Option<String>,
) -> Result<String, String> {
    match Ic2P2ramp::deposit_funds(chain_id, amount, token_address, gas).await {
        Ok(_) => Ok("Funds deposited successfully".to_string()),
        Err(err) => Err(format!("Failed to deposit funds: {}", err)),
    }
}

#[ic_cdk::update]
async fn test_check_and_approve_token(
    chain_id: u64,
    token_address: String,
) -> Result<bool, String> {
    let fee_estimates = fees::get_fee_estimates(9, chain_id).await;
    Ic2P2ramp::check_and_approve_token(chain_id, token_address, U256::from(21_000), fee_estimates)
        .await
}

// ---------
// XRC Rate
// ---------

#[ic_cdk::update]
async fn get_usd_exchange_rate(
    fiat_symbol: String,
    crypto_symbol: String,
) -> Result<String, String> {
    match xrc_rates::get_exchange_rate(&fiat_symbol, &crypto_symbol).await {
        Ok(rate) => Ok(rate.to_string()),
        Err(err) => Err(err),
    }
}

// -----
// USERS
// -----

#[ic_cdk::update]
fn register_user(
    evm_address: String,
    payment_providers: Vec<PaymentProvider>,
) -> Result<String, String> {
    user_management::register_user(evm_address, payment_providers)
}

#[ic_cdk::query]
fn get_user(evm_address: String) -> Result<User, String> {
    user_management::get_user(evm_address)
}

#[ic_cdk::query]
fn add_payment_provider(
    evm_address: String,
    payment_provider: PaymentProvider,
) -> Result<String, String> {
    user_management::add_payment_provider(evm_address, payment_provider)
}

// ------------------
// ICP Offramp Orders
// ------------------

#[ic_cdk::query]
fn get_orders() -> Vec<Order> {
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
    token_type: String,
) -> Result<String, String> {
    // evm::vault::deposit_funds(chain_id, crypto_amount, token_type.clone()).await?;

    order_management::create_order(
        fiat_amount,
        fiat_symbol,
        crypto_amount,
        offramper_providers,
        address,
        chain_id,
        token_type,
    )
}

#[ic_cdk::update]
fn lock_order(
    order_id: String,
    onramper_provider: PaymentProvider,
    onramper_address: String,
) -> Result<String, String> {
    // let order = management::get_order_by_id(order_id.clone()).await?;
    // evm::vault::commit_order(order.chain_id, order.offramper_address, order.crypto_amount).await?;

    order_management::lock_order(order_id, onramper_provider, onramper_address)
}

#[ic_cdk::update]
fn remove_order(order_id: String) -> Result<String, String> {
    // let order = management::get_order_by_id(order_id.clone()).await?;

    // evm::vault::withdraw(order.chain_id, order.crypto_amount).await?;

    order_management::remove_order(order_id)
}

// ---------------
// Paypal Payment
// ---------------

#[ic_cdk::update]
async fn verify_transaction(
    order_id: String,
    chain_id: u64,
    transaction_id: String,
) -> Result<String, String> {
    let access_token = paypal_auth::get_paypal_access_token().await?;
    let cycles: u128 = 10_000_000_000;
    let order = order_management::get_order_by_id(order_id.clone())?;

    let capture_details =
        paypal_capture::fetch_paypal_capture_details(&access_token, &transaction_id, cycles)
            .await?;

    // Verify the captured payment details
    let expected_fiat_amount = order.fiat_amount as f64 / 100.0; // Assuming fiat_amount is in cents
    let received_amount: f64 = capture_details
        .amount
        .value
        .parse()
        .map_err(|e| format!("Failed to parse amount: {}", e))?;
    ic_cdk::println!("received_amount = {}", received_amount);

    let amount_matches = (received_amount - expected_fiat_amount).abs() < f64::EPSILON;
    let currency_matches = capture_details.amount.currency_code == order.currency_symbol;
    let offramper_matches =
        capture_details.payee.email_address == order.offramper_providers[0].get_id();
    // let onramper_matches = order.onramper_paypal_id.as_deref()
    //     == Some(&capture_details.supplementary_data.related_ids.order_id);

    if capture_details.status == "COMPLETED"
        && amount_matches
        && currency_matches
        && offramper_matches
    // && onramper_matches
    {
        // Update the order status in your storage
        order_management::mark_order_as_paid(order.id)?;
        Ic2P2ramp::release_base_currency(chain_id.into(), order_id).await?;
        Ok("Payment verified successfully".to_string())
    } else {
        Err("Payment verification failed".to_string())
    }
}

ic_cdk::export_candid!();
