use crate::{
    errors::{Result, SystemError},
    model::errors::OrderError,
    outcalls::stripe::session::retrieve_session,
};

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
