mod evm;
mod order;
mod outcalls;
mod state;

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use evm::rpc::{RpcApi, RpcServices};
use order::management;
use outcalls::{paypal_auth, xrc_rates};
use state::storage::Order;
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
    println!("[init]: initialized minter with arg: {:?}", arg);
    initialize_state(state::State::try_from(arg).expect("BUG: failed to initialize minter"));
    setup_timers();
}

#[ic_cdk::query]
fn get_evm_address() -> String {
    read_state(|s| s.evm_address.clone()).expect("evm address should be initialized")
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

// ---------------
// Paypal Payment
// ---------------

#[derive(Serialize, Deserialize, Debug)]
struct PayPalCaptureDetails {
    id: String,
    status: String,
    amount: Amount,
}

#[derive(Serialize, Deserialize, Debug)]
struct Amount {
    currency_code: String,
    value: String,
}

#[ic_cdk::update]
async fn verify_transaction(order_id: String, transaction_id: String) -> Result<String, String> {
    let access_token = paypal_auth::get_paypal_access_token().await?;

    let url = format!(
        "https://api-m.sandbox.paypal.com/v2/payments/captures/{}",
        transaction_id
    );

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    let order = management::get_order_by_id(order_id.clone()).await?;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");
            ic_cdk::println!("str_body = {:?}", str_body);

            let capture_details: PayPalCaptureDetails = serde_json::from_str(&str_body)
                .map_err(|e| format!("Failed to parse response: {}", e))?;
            ic_cdk::println!("capture_details = {:?}", capture_details);

            // Verify the captured payment details
            let expected_fiat_amount = order.fiat_amount as f64 / 100.0; // Assuming fiat_amount is in cents
            let received_amount: f64 = capture_details
                .amount
                .value
                .parse()
                .map_err(|e| format!("Failed to parse amount: {}", e))?;

            ic_cdk::println!("received_amount = {}", received_amount);
            if capture_details.status == "COMPLETED"
                && (received_amount - expected_fiat_amount).abs() < f64::EPSILON
            {
                // Update the order status in your storage
                management::mark_order_as_paid(order.id).await?;
                evm::vault::release_base_currency(order_id).await?;
                Ok("Payment verified successfully".to_string())
            } else {
                Err("Payment verification failed".to_string())
            }
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            Err(message)
        }
    }
}

// ------------------
// ICP Offramp Orders
// ------------------

#[ic_cdk::query]
fn get_orders() -> Vec<Order> {
    management::get_orders()
}

#[ic_cdk::update]
async fn create_order(
    fiat_amount: u64,
    crypto_amount: u64,
    paypal_id: String,
    address: String,
    chain_id: u64,
    token_type: String,
) -> Result<String, String> {
    // evm::vault::deposit_funds(chain_id, crypto_amount, token_type.clone()).await?;

    management::create_order(
        fiat_amount,
        crypto_amount,
        paypal_id,
        address,
        chain_id,
        token_type,
    )
    .await
}

#[ic_cdk::update]
async fn lock_order(
    order_id: String,
    onramper_paypal_id: String,
    onramper_address: String,
) -> Result<String, String> {
    // let order = management::get_order_by_id(order_id.clone()).await?;
    // evm::vault::commit_order(order.chain_id, order.offramper_address, order.crypto_amount).await?;

    management::lock_order(order_id, onramper_paypal_id, onramper_address).await
}

#[ic_cdk::update]
async fn remove_order(order_id: String) -> Result<String, String> {
    // let order = management::get_order_by_id(order_id.clone()).await?;

    // evm::vault::withdraw(order.chain_id, order.crypto_amount).await?;

    management::remove_order(order_id).await
}

// #[ic_cdk::update]
// async fn submit_payment_proof(
//     order_id: String,
//     proof: Vec<u8>,
//     chain_id: u64,
// ) -> Result<String, String> {
//     let is_valid_proof = verifier::verify_payment_proof(order_id.clone(), proof, chain_id).await;

//     if is_valid_proof {
//         let order = management::get_order_by_id(order_id.clone()).await?;
//         let result = evm::vault::release_funds(
//             chain_id,
//             order
//                 .onramper_address
//                 .expect("onramper address not specified"),
//             order.crypto_amount,
//         )
//         .await;

//         match result {
//             Ok(_) => storage::ORDERS.with(|orders| {
//                 let mut orders = orders.borrow_mut();
//                 if let Some(mut order) = orders.remove(&order_id) {
//                     order.proof_submitted = true;
//                     order.payment_done = true;
//                     order.removed = true;
//                     orders.insert(order_id.clone(), order);
//                     Ok("Payment proof verified and funds released".to_string())
//                 } else {
//                     Err("Order not found".to_string())
//                 }
//             }),
//             Err(err) => Err(err),
//         }
//     } else {
//         Err("Invalid payment proof".to_string())
//     }
// }

ic_cdk::export_candid!();
