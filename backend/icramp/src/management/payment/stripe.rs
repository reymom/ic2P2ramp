use crate::{
    errors::{Result, SystemError},
    model::{
        errors::OrderError,
        memory::stable::orders::append_fill_if_new,
        types::{
            PaymentProvider, PaymentProviderType,
            orders::{FillRecord, LockedOrder},
        },
    },
    outcalls::stripe::session::retrieve_session,
};

pub async fn verify_stripe_payment(order: &LockedOrder, email: &str) -> Result<()> {
    // 1. Stored Checkout Session id from lock()
    let session_id = order
        .payment_id
        .clone()
        .ok_or(OrderError::PaymentVerificationFailed)?;

    // 2. Stripe platform (offramper's Connect platform)
    let platform = order
        .base
        .offramper_providers
        .iter()
        .find_map(|p| {
            if let PaymentProvider::Stripe { platform, .. } = p.1 {
                Some(platform.clone())
            } else {
                None
            }
        })
        .ok_or(OrderError::PaymentVerificationFailed)?;

    let expected_minor = (order.price + order.offramper_fee) as i64;
    let stripe_account_id = order
        .base
        .offramper_providers
        .get(&PaymentProviderType::Stripe)
        .and_then(|provider| {
            if let PaymentProvider::Stripe { account_id, .. } = provider {
                Some(account_id)
            } else {
                None
            }
        })
        .ok_or_else(|| OrderError::InvalidOfframperProvider)?;
    let correct = verify_session_paid_destination(
        &session_id,
        expected_minor,
        &order.base.currency,
        &stripe_account_id,
        Some(platform),
        email,
    )
    .await?;

    if correct {
        ic_cdk::println!("[verify_transaction] Verification succeded.");
        crate::management::order::mark_order_as_paid(order.base.id)?;
    } else {
        return Err(OrderError::PaymentVerificationFailed)?;
    }

    let total = order.base.crypto.amount.max(1);
    let locked = order.lock_amount;
    let fee_part: u128 = (order.base.crypto.fee.saturating_mul(locked)) / total;
    append_fill_if_new(
        order.base.id,
        FillRecord {
            payer_user_id: order.onramper.user_id,
            payer: order.onramper.address.clone(),
            provider: order.onramper.provider.clone(),
            fiat: order.price,
            offramper_fee: order.offramper_fee,
            crypto_amount: locked,
            crypto_fee: fee_part,
            payment_id: order.payment_id.clone().unwrap_or("".to_string()),
            tx_id: None,
            created_at: ic_cdk::api::time(),
        },
    )
}

// Verify success against order + destination
pub async fn verify_session_paid_destination(
    session_id: &str,
    expected_minor: i64,
    expected_currency_upper: &str, // e.g. "EUR"
    expected_destination: &str,    // acct_...
    platform_label: Option<String>,
    onramper_email: &str,
) -> Result<bool> {
    let s = retrieve_session(session_id, true, platform_label).await?;
    let paid = s.payment_status.as_deref() == Some("paid");

    // pull fields from expanded PI
    let pi = s
        .payment_intent
        .as_ref()
        .and_then(|v| v.as_object())
        .ok_or_else(|| SystemError::ParseError("no payment_intent".into()))?;

    let pi_status = pi.get("status").and_then(|v| v.as_str());
    let currency = pi
        .get("currency")
        .and_then(|v| v.as_str())
        .map(|c| c.to_uppercase());
    let dest = pi
        .get("transfer_data")
        .and_then(|x| x.get("destination"))
        .and_then(|v| v.as_str());
    let amount_received = pi
        .get("amount_received")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let payer_email = s
        .customer_details
        .as_ref()
        .and_then(|d| d.email.as_deref())
        .or(s.customer_email.as_deref())
        .or_else(|| {
            s.payment_intent
                .as_ref()
                .and_then(|pi| pi.pointer("/charges/data/0/billing_details/email"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("");
    if !payer_email
        .trim()
        .eq_ignore_ascii_case(onramper_email.trim())
    {
        return Err(OrderError::PaymentVerificationFailed)?;
    }

    let ok = paid
        && pi_status == Some("succeeded")
        && currency.as_deref() == Some(expected_currency_upper)
        && dest == Some(expected_destination)
        && amount_received >= expected_minor;

    Ok(ok)
}
