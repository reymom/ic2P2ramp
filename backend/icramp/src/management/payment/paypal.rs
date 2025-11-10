use crate::{
    model::{
        errors::{OrderError, Result},
        memory::stable::orders::append_fill_if_new,
        types::{
            PaymentProvider, PaymentProviderType,
            orders::{FillRecord, LockedOrder},
        },
    },
    outcalls::paypal,
};

pub async fn verify_paypal_payment(
    onramper_id: &str,
    transaction_id: &str,
    order: &LockedOrder,
) -> Result<()> {
    let access_token = paypal::auth::get_paypal_access_token().await?;
    ic_cdk::println!("[verify_transaction] Obtained PayPal access token");
    let capture_details = paypal::order::fetch_paypal_order(&access_token, transaction_id).await?;

    let received_amount: f64 = capture_details
        .purchase_units
        .iter()
        .flat_map(|unit| &unit.payments.captures)
        .map(|capture| capture.amount.value.parse::<f64>().unwrap())
        .sum();

    let amount_matches = order.payment_amount_matches(&received_amount.to_string());
    let currency_matches =
        capture_details.purchase_units[0].amount.currency_code == order.base.currency;

    let offramper_provider = order
        .base
        .offramper_providers
        .iter()
        .find(|(provider_type, _)| *provider_type == &PaymentProviderType::PayPal)
        .ok_or(OrderError::InvalidOfframperProvider)?;

    let PaymentProvider::PayPal { id: offramper_id } = offramper_provider.1 else {
        return Err(OrderError::InvalidOfframperProvider)?;
    };

    let offramper_matches = capture_details.purchase_units[0].payee.email_address == *offramper_id;
    let onramper_matches = capture_details.payer.email_address == *onramper_id;

    if capture_details.status == "COMPLETED"
        && amount_matches
        && currency_matches
        && offramper_matches
        && onramper_matches
    {
        ic_cdk::println!("[verify_transaction] Verification succeded.");
        crate::memory::stable::orders::set_payment_id(order.base.id, transaction_id.to_string())?;
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
            payment_id: transaction_id.to_string(),
            tx_id: None,
            created_at: ic_cdk::api::time(),
        },
    )
}
