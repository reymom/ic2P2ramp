use std::collections::HashMap;

use evm_rpc_canister_types::BlockTag;

use crate::{
    evm::vault::Ic2P2ramp,
    management::payment::stripe as stripe_mgmt,
    model::{
        errors::{Result, SystemError},
        types::{
            evm::{gas::ChainGasTracking, transaction::TransactionAction},
            exchange_rate::ExchangeRateCache,
        },
    },
    outcalls::{paypal, revolut, stripe},
};

#[ic_cdk::update]
async fn test_estimate_gas_commit(
    chain_id: u64,
    offramper: String,
    token_address: Option<String>,
    amount: u128,
) -> Result<Option<u64>> {
    let commit_inputs = Ic2P2ramp::commit_inputs(offramper, token_address, amount)?;
    let transaction_type = TransactionAction::Commit;
    let (vault, data) =
        crate::evm::helper::get_vault_and_data(chain_id, &transaction_type, &commit_inputs)?;

    Ic2P2ramp::estimate_gas(chain_id, vault, data, None).await
}

#[ic_cdk::update]
async fn test_get_fee_estimates(chain_id: u64) -> Result<(u128, u128)> {
    crate::fees::get_fee_estimates(9, chain_id)
        .await
        .map(|fees| {
            (
                fees.max_fee_per_gas.as_u128(),
                fees.max_priority_fee_per_gas.as_u128(),
            )
        })
}

#[ic_cdk::update]
async fn test_get_latest_block(chain_id: u64) -> Result<candid::Nat> {
    crate::fees::eth_get_latest_block(chain_id, BlockTag::Latest)
        .await
        .map(|block| block.number)
}

#[ic_cdk::update]
async fn test_get_latest_nonce(chain_id: u64) -> Result<candid::Nat> {
    crate::fees::eth_get_latest_block(chain_id, BlockTag::Latest)
        .await
        .map(|block| block.nonce)
}

#[ic_cdk::update]
async fn test_paypal() -> Result<String> {
    paypal::auth::get_paypal_access_token().await
}

#[ic_cdk::query]
async fn test_get_gas_tracking(chain_id: u64) -> Result<ChainGasTracking> {
    crate::gas::get_gas_tracking(chain_id)
}

#[ic_cdk::query]
async fn test_get_rates() -> HashMap<(String, String), ExchangeRateCache> {
    crate::memory::heap::tmp_get_rate()
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

    revolut::authorize::get_authorization_url(&consent_id).await
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

// ------
// Stripe
// ------

// Inspect connected account JSON (capabilities/region)
#[ic_cdk::update]
async fn stripe_test_get_raw_account(
    account_id: String,
    platform_label: Option<String>,
) -> Result<String> {
    stripe::account::get_account_raw(&account_id, platform_label).await
}

// (Optional) Dashboard login link for debugging (returns URL)
#[ic_cdk::update]
async fn stripe_test_create_login_link(
    account_id: String,
    platform_label: Option<String>,
) -> Result<String> {
    stripe::account::create_login_link(&account_id, platform_label).await
}

// Create a Checkout Session (destination charge) and get (session_id, url)
#[ic_cdk::update]
async fn stripe_test_create_session(
    amount_minor: u64,        // e.g., 1999 for €19.99
    currency_upper: String,   // "EUR"
    destination_acct: String, // acct_...
    platform_label: Option<String>,
    success_url: String,
    cancel_url: String,
    payer_email: String,
) -> Result<(String, String)> {
    stripe::session::create_checkout_session_for_order(
        0, // not used in test; only in the product name string
        &destination_acct,
        amount_minor,
        &currency_upper,
        platform_label,
        success_url,
        cancel_url,
        payer_email,
    )
    .await
}

// Retrieve a session (expanded) as JSON
#[ic_cdk::update]
async fn stripe_test_retrieve_session(
    session_id: String,
    expand_pi: bool,
    platform_label: Option<String>,
) -> Result<String> {
    let s = stripe::session::retrieve_session(&session_id, expand_pi, platform_label).await?;
    Ok(serde_json::to_string(&s).map_err(|e| SystemError::ParseError(e.to_string()))?)
}

// Verify paid + destination match → bool
#[ic_cdk::update]
async fn stripe_test_verify_session(
    session_id: String,
    expected_minor: i64,
    expected_currency_upper: String,
    expected_destination: String,
    platform_label: Option<String>,
    payer_email: String,
) -> Result<bool> {
    stripe_mgmt::verify_session_paid_destination(
        &session_id,
        expected_minor,
        &expected_currency_upper,
        &expected_destination,
        platform_label,
        &payer_email,
    )
    .await
}
