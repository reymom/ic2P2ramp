mod errors;
mod evm;
mod management;
mod outcalls;
mod state;

use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use state::blockchain::Blockchain;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

use errors::{RampError, Result};
use evm::transaction::spawn_transaction_checker;
use evm::{helpers, providers, rpc::ProviderView, vault::Ic2P2ramp};
use management::order as order_management;
use management::user as user_management;
use outcalls::{paypal, revolut, xrc_rates};
use state::storage::{
    self, Address, OrderFilter, OrderState, PaymentProvider, PaymentProviderType, User, UserType,
};
use state::{contains_provider_type, initialize_state, mutate_state, read_state, InitArg};

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

// #[ic_cdk::pre_upgrade]
// fn pre_upgrade() {
//     fn pre_upgrade() {
//         let order_id_counter = ORDER_ID_COUNTER.with(|counter| *counter.borrow());
//         let locked_order_timers = LOCKED_ORDER_TIMERS.with(|timers| {
//             timers
//                 .borrow()
//                 .iter()
//                 .map(|(&order_id, _)| {
//                     (order_id, ic_cdk::api::time() + 3600) // Example: Timer set for 1 hour from current time
//                 })
//                 .collect::<HashMap<u64, u64>>()
//         });
//         let serializable_state = SerializableState {
//             order_id_counter,
//             locked_order_timers,
//         };
//         let mut state_bytes = vec![];
//         ciborium::ser::into_writer(&serializable_state, &mut state_bytes)
//             .expect("failed to encode state");
//         let len = state_bytes.len() as u32;
//         let mut memory = memory::get_upgrades_memory();
//         let mut writer = Writer::new(&mut memory, 0);
//         writer.write(&len.to_le_bytes()).unwrap();
//         writer.write(&state_bytes).unwrap();
//     }
// }

// #[ic_cdk::post_upgrade]
// fn post_upgrade() {
//     let memory = memory::get_upgrades_memory();
//     let mut state_len_bytes = [0; 4];
//     memory.read(0, &mut state_len_bytes);
//     let state_len = u32::from_le_bytes(state_len_bytes) as usize;
//     let mut state_bytes = vec![0; state_len];
//     memory.read(4, &mut state_bytes);
//     let serializable_state: SerializableState =
//         ciborium::de::from_reader(&*state_bytes).expect("failed to decode state");
//     ORDER_ID_COUNTER.with(|counter| {
//         *counter.borrow_mut() = serializable_state.order_id_counter;
//     });
//     LOCKED_ORDER_TIMERS.with(|timers| {
//         let mut timers = timers.borrow_mut();
//         for (order_id, expiration_time) in serializable_state.locked_order_timers {
//             let remaining_duration = expiration_time - ic_cdk::api::time();
//             if remaining_duration > 0 {
//                 let timer_id = set_timer(Duration::from_secs(remaining_duration), move || {
//                     ic_cdk::spawn(async move {
//                         if let Err(e) = management::order::unlock_order(order_id) {
//                             ic_cdk::println!("Failed to auto-unlock order {}: {:?}", order_id, e);
//                         }
//                     });
//                 });
//                 timers.insert(order_id, timer_id);
//             }
//         }
//     });
// }

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
async fn transfer_value(
    chain_id: u64,
    to: String,
    amount: u128,
    gas: Option<i32>,
) -> Result<String> {
    helpers::validate_evm_address(&to)?;
    Ic2P2ramp::transfer_eth(chain_id, to, amount, gas).await
}

// -----
// Tests
// -----

#[ic_cdk::update]
async fn test_deposit_funds(
    chain_id: u64,
    amount: u64,
    token_address: Option<String>,
    gas: Option<u32>,
) -> Result<String> {
    Ic2P2ramp::deposit_funds(chain_id, amount, token_address, gas).await
}

#[ic_cdk::update]
async fn test_get_revolut_token() -> Result<String> {
    Ok(revolut::auth::get_revolut_access_token().await?)
}

#[ic_cdk::update]
async fn test_get_paypal_token() -> Result<String> {
    Ok(paypal::auth::get_paypal_access_token().await?)
}

// ----------
// Management
// ----------

#[ic_cdk::update]
async fn approve_token_allowance(chain_id: u64, token_address: String, gas: u32) -> Result<()> {
    Ic2P2ramp::approve_token_allowance(chain_id, &token_address, gas).await
}

#[ic_cdk::query]
async fn get_rpc_providers() -> Vec<ProviderView> {
    providers::get_providers().await
}

// ---------
// XRC Rate
// ---------

#[ic_cdk::update]
async fn get_exchange_rate(fiat_symbol: String, crypto_symbol: String) -> Result<String> {
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
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
    login_address: Address,
) -> Result<User> {
    user_management::register_user(user_type, payment_providers, login_address)
}

#[ic_cdk::query]
fn get_user(address: Address) -> Result<User> {
    storage::get_user(&address)
}

#[ic_cdk::update]
fn remove_user(address: Address) -> Result<User> {
    storage::remove_user(&address)
}

#[ic_cdk::update]
fn add_address_for_user(login_address: Address, address: Address) -> Result<()> {
    user_management::add_address(&login_address, address)
}

#[ic_cdk::update]
fn add_payment_provider_for_user(
    address: Address,
    payment_provider: PaymentProvider,
) -> Result<()> {
    user_management::add_payment_provider(&address, payment_provider)
}

// ------------------
// ICP Offramp Orders
// ------------------

#[ic_cdk::query]
fn get_orders(filter: Option<OrderFilter>) -> Vec<OrderState> {
    order_management::get_orders(filter)
}

#[ic_cdk::update]
fn create_order(
    fiat_amount: u64,
    fiat_symbol: String,
    offramper_providers: HashMap<PaymentProviderType, String>,
    blockchain: Blockchain,
    token_address: Option<String>,
    crypto_amount: u64,
    offramper_address: Address,
) -> Result<u64> {
    let user = storage::get_user(&offramper_address)?;
    user.is_banned()?;
    user.validate_offramper()?;

    for (provider_type, provider_id) in &offramper_providers {
        let provider = PaymentProvider {
            provider_type: provider_type.clone(),
            id: provider_id.clone(),
        };

        if !user.payment_providers.contains(&provider) {
            return Err(RampError::ProviderNotInUser(provider_type.clone()));
        }
    }

    order_management::create_order(
        fiat_amount,
        fiat_symbol,
        offramper_providers,
        blockchain,
        token_address,
        crypto_amount,
        offramper_address,
    )
}

#[ic_cdk::update]
async fn lock_order(
    order_id: u64,
    onramper_provider: PaymentProvider,
    onramper_address: Address,
    gas: Option<u32>,
) -> Result<String> {
    user_management::can_commit_orders(&onramper_address)?;

    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Created(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    if !contains_provider_type(&onramper_provider, &order.offramper_providers) {
        return Err(RampError::InvalidOnramperProvider);
    }

    match order.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let tx_hash = Ic2P2ramp::commit_deposit(
                chain_id,
                order.offramper_address.address,
                order.crypto.token,
                order.crypto.amount,
                gas,
            )
            .await?;
            spawn_transaction_checker(
                tx_hash.clone(),
                chain_id,
                60,
                Duration::from_secs(4),
                move || {
                    let _ = order_management::lock_order(
                        order_id,
                        onramper_provider.clone(),
                        onramper_address.clone(),
                    );
                },
            );
            Ok(tx_hash)
        }
        _ => todo!(),
    }
}

#[ic_cdk::update]
async fn unlock_order(order_id: u64, gas: Option<u32>) -> Result<String> {
    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    match order.base.crypto.blockchain {
        Blockchain::EVM { chain_id } => {
            let tx_hash = Ic2P2ramp::uncommit_deposit(
                chain_id,
                order.base.offramper_address.address,
                order.base.crypto.token,
                order.base.crypto.amount,
                gas,
            )
            .await?;
            spawn_transaction_checker(
                tx_hash.clone(),
                chain_id,
                60,
                Duration::from_secs(4),
                move || {
                    let _ = order_management::unlock_order(order_id);
                },
            );
            Ok(tx_hash)
        }
        _ => todo!(),
    }
}

#[ic_cdk::update]
fn cancel_order(order_id: u64) -> Result<()> {
    order_management::cancel_order(order_id)
}

// ---------------
// Paypal Payment
// ---------------

#[ic_cdk::update]
async fn verify_transaction(order_id: u64, transaction_id: String, gas: Option<u32>) -> Result<()> {
    ic_cdk::println!(
        "[verify_transaction] Starting verification for order ID: {} and transaction ID: {}",
        order_id,
        transaction_id
    );

    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let cycles: u128 = 10_000_000_000;
    let access_token = paypal::auth::get_paypal_access_token().await?;
    ic_cdk::println!("[verify_transaction] Obtained PayPal access token");
    let capture_details =
        paypal::order::fetch_paypal_order(&access_token, &transaction_id, cycles).await?;

    // Verify the captured payment details (amounts are in cents)
    let total_expected_amount = (order.base.fiat_amount + order.base.offramper_fee) as f64 / 100.0;

    let received_amount: f64 = capture_details
        .purchase_units
        .iter()
        .flat_map(|unit| &unit.payments.captures)
        .map(|capture| capture.amount.value.parse::<f64>().unwrap())
        .sum();
    ic_cdk::println!("received_amount = {}", received_amount);

    let amount_matches = (received_amount - total_expected_amount).abs() < f64::EPSILON;
    let currency_matches =
        capture_details.purchase_units[0].amount.currency_code == order.base.currency_symbol;

    let offramper_provider_id = order
        .base
        .offramper_providers
        .get(&order.onramper_provider.provider_type)
        .ok_or_else(|| RampError::ProviderNotInUser(order.onramper_provider.provider_type))?;

    let offramper_matches =
        capture_details.purchase_units[0].payee.email_address == *offramper_provider_id;
    let onramper_matches = capture_details.payer.email_address == order.onramper_provider.id;

    if capture_details.status == "COMPLETED"
        && amount_matches
        && currency_matches
        && offramper_matches
        && onramper_matches
    {
        ic_cdk::println!("[verify_transaction] verified is true!!");
        order_management::mark_order_as_paid(order.base.id)?;

        match order.base.crypto.blockchain {
            Blockchain::EVM { chain_id } => {
                let tx_hash = Ic2P2ramp::release_funds(order_id, chain_id, gas).await?;
                spawn_transaction_checker(
                    tx_hash,
                    chain_id,
                    60,
                    Duration::from_secs(4),
                    move || {
                        // Update order state to completed
                        match management::order::set_order_completed(order_id) {
                            Ok(_) => {
                                ic_cdk::println!(
                                    "[verify_transaction] order {:?} completed",
                                    order_id
                                )
                            }
                            Err(e) => {
                                ic_cdk::trap(format!("could not complete order: {:?}", e).as_str())
                            }
                        }
                    },
                );
            }
            _ => todo!(),
        }

        Ok(())
    } else {
        Err(RampError::PaymentVerificationFailed)
    }
}

ic_cdk::export_candid!();
