use crate::{
    model::{
        errors::{OrderError, Result, UserError},
        memory::stable::orders::append_fill_if_new,
        types::{
            PaymentProvider, PaymentProviderType, find_provider_of_type,
            orders::{FillRecord, LockedOrder, RevolutConsent},
        },
    },
    outcalls::revolut,
};

pub async fn verify_revolut_payment(
    onramper_id: &str,
    onramper_scheme: &str,
    transaction_id: &str,
    order: &LockedOrder,
) -> Result<()> {
    let payment_details =
        revolut::transaction::fetch_revolut_payment_details(transaction_id).await?;

    // Verify the captured payment details (amounts are in cents)
    let amount_matches =
        order.payment_amount_matches(&payment_details.data.initiation.instructed_amount.amount);
    let currency_matches =
        payment_details.data.initiation.instructed_amount.currency == order.base.currency;

    let onramper_account = match payment_details.data.initiation.debtor_account {
        Some(details) => details,
        None => return Err(OrderError::MissingDebtorAccount)?,
    };
    let debtor_matches = onramper_account.scheme_name == *onramper_scheme
        && onramper_account.identification == *onramper_id;

    let offramper_account = payment_details.data.initiation.creditor_account;

    let off_provider = find_provider_of_type(
        &order.base.offramper_providers,
        PaymentProviderType::Revolut,
    )
    .ok_or(UserError::ProviderNotInUser(PaymentProviderType::Revolut))?;

    let PaymentProvider::Revolut {
        scheme: offramper_scheme,
        id: offramper_id,
        name: offramper_name,
    } = off_provider
    else {
        return Err(OrderError::InvalidOfframperProvider)?;
    };

    let creditor_matches = offramper_account.scheme_name == *offramper_scheme
        && offramper_account.identification == *offramper_id
        && offramper_account.name == *offramper_name;

    if payment_details.data.status == "AcceptedSettlementCompleted"
        && amount_matches
        && currency_matches
        && debtor_matches
        && creditor_matches
    {
        ic_cdk::println!("[verify_transaction] verified is true!!");
        crate::management::order::mark_order_as_paid(order.base.id)?;
    } else {
        Err(OrderError::PaymentVerificationFailed)?
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

pub async fn get_revolut_consent(
    offramper_providers: Vec<PaymentProvider>,
    fiat_amount: &str,
    currency_symbol: &str,
    onramper_provider: &PaymentProvider,
) -> Result<Option<RevolutConsent>> {
    match &onramper_provider {
        PaymentProvider::Revolut {
            scheme: onramper_scheme,
            id: onramper_id,
            ..
        } => {
            let off_provider =
                find_provider_of_type(&offramper_providers, PaymentProviderType::Revolut)
                    .ok_or(UserError::ProviderNotInUser(PaymentProviderType::Revolut))?;

            if let PaymentProvider::Revolut {
                scheme: offramper_scheme,
                id: offramper_id,
                name: offramper_name,
            } = off_provider
            {
                let consent_id = revolut::consent::create_account_access_consent(
                    fiat_amount,
                    currency_symbol,
                    onramper_scheme,
                    onramper_id,
                    offramper_scheme,
                    offramper_id,
                    &offramper_name
                        .clone()
                        .ok_or(OrderError::InvalidOfframperProvider)?,
                )
                .await?;

                let auth_url = revolut::authorize::get_authorization_url(&consent_id).await?;
                Ok(Some(RevolutConsent::new(consent_id, auth_url)))
            } else {
                Err(OrderError::InvalidOrderState("Expected Revolut provider".to_string()).into())
            }
        }
        _ => Ok(None),
    }
}
