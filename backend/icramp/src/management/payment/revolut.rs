use std::collections::HashMap;

use crate::{
    model::{
        errors::{OrderError, Result, UserError},
        types::{
            PaymentProvider, PaymentProviderType,
            orders::{LockedOrder, RevolutConsent},
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

    let offramper_provider = order
        .base
        .offramper_providers
        .iter()
        .find(|(provider_type, _)| *provider_type == &PaymentProviderType::Revolut)
        .ok_or(OrderError::InvalidOfframperProvider)?;

    let PaymentProvider::Revolut {
        scheme: offramper_scheme,
        id: offramper_id,
        name: offramper_name,
    } = offramper_provider.1
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
        crate::management::order::mark_order_as_paid(order.base.id)
    } else {
        Err(OrderError::PaymentVerificationFailed)?
    }
}

pub async fn get_revolut_consent(
    offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
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
            let offramper_provider = offramper_providers
                .get(&PaymentProviderType::Revolut)
                .ok_or(UserError::ProviderNotInUser(PaymentProviderType::Revolut))?;

            if let PaymentProvider::Revolut {
                scheme: offramper_scheme,
                id: offramper_id,
                name: offramper_name,
            } = offramper_provider
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
