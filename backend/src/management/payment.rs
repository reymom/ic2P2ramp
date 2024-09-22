use std::collections::HashMap;

use candid::Principal;
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use crate::{
    evm::vault::Ic2P2ramp,
    icp::vault::Ic2P2ramp as ICPRamp,
    management,
    model::{
        errors::{RampError, Result},
        memory::stable::orders,
        types::{
            evm::gas::MethodGasUsage,
            icp::get_icp_token,
            order::{LockedOrder, OrderState, RevolutConsent},
            PaymentProvider, PaymentProviderType,
        },
    },
    outcalls::{paypal, revolut},
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
        .ok_or(RampError::InvalidOfframperProvider)?;

    let PaymentProvider::PayPal { id: offramper_id } = offramper_provider.1 else {
        return Err(RampError::InvalidOfframperProvider);
    };

    let offramper_matches = capture_details.purchase_units[0].payee.email_address == *offramper_id;
    let onramper_matches = capture_details.payer.email_address == *onramper_id;

    if capture_details.status == "COMPLETED"
        && amount_matches
        && currency_matches
        && offramper_matches
        && onramper_matches
    {
        ic_cdk::println!("[verify_transaction] verified is true!!");
        management::order::set_payment_id(order.base.id, transaction_id.to_string())?;
        management::order::mark_order_as_paid(order.base.id)?;
    } else {
        return Err(RampError::PaymentVerificationFailed);
    }

    Ok(())
}

pub async fn verify_revolut_payment(
    onramper_id: &str,
    onramper_scheme: &str,
    transaction_id: &str,
    order: &LockedOrder,
) -> Result<()> {
    let payment_details =
        revolut::transaction::fetch_revolut_payment_details(&transaction_id).await?;

    // Verify the captured payment details (amounts are in cents)
    let amount_matches =
        order.payment_amount_matches(&payment_details.data.initiation.instructed_amount.amount);
    let currency_matches =
        payment_details.data.initiation.instructed_amount.currency == order.base.currency;

    let onramper_account = match payment_details.data.initiation.debtor_account {
        Some(details) => details,
        None => return Err(RampError::MissingDebtorAccount),
    };
    let debtor_matches = onramper_account.scheme_name == *onramper_scheme
        && onramper_account.identification == *onramper_id;

    let offramper_account = payment_details.data.initiation.creditor_account;

    let offramper_provider = order
        .base
        .offramper_providers
        .iter()
        .find(|(provider_type, _)| *provider_type == &PaymentProviderType::Revolut)
        .ok_or(RampError::InvalidOfframperProvider)?;

    let PaymentProvider::Revolut {
        scheme: offramper_scheme,
        id: offramper_id,
        name: offramper_name,
    } = offramper_provider.1
    else {
        return Err(RampError::InvalidOfframperProvider);
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
        management::order::mark_order_as_paid(order.base.id)
    } else {
        return Err(RampError::PaymentVerificationFailed);
    }
}

pub async fn handle_evm_payment_completion(
    order_id: u64,
    chain_id: u64,
    gas: Option<u64>,
) -> Result<String> {
    let order_state = orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };
    let mut action_type = MethodGasUsage::ReleaseNative;
    if let Some(_) = order.base.crypto.token {
        action_type = MethodGasUsage::ReleaseToken
    };
    let (tx_hash, sign_request) = Ic2P2ramp::release_funds(order, chain_id, gas).await?;

    management::vault::spawn_payment_release(
        order_id,
        chain_id,
        action_type,
        &tx_hash,
        sign_request,
    );
    Ok(tx_hash)
}

pub async fn handle_icp_payment_completion(
    order_id: u64,
    ledger_principal: &Principal,
) -> Result<String> {
    let order_state = orders::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let onramper_principal = Principal::from_text(&order.onramper.address.address).unwrap();

    let amount = NumTokens::from(order.base.crypto.amount);
    let fee = get_icp_token(ledger_principal)?.fee;

    let to_account = Account {
        owner: onramper_principal,
        subaccount: None,
    };
    let index = ICPRamp::transfer(
        *ledger_principal,
        to_account,
        amount - order.base.crypto.fee,
        Some(fee),
    )
    .await?;

    super::order::set_order_completed(order_id)?;

    Ok(index.to_string())
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
                .ok_or_else(|| RampError::ProviderNotInUser(PaymentProviderType::Revolut))?;

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
                    &offramper_scheme,
                    &offramper_id,
                    &offramper_name
                        .clone()
                        .ok_or_else(|| RampError::InvalidOfframperProvider)?,
                )
                .await?;

                let auth_url = revolut::authorize::get_authorization_url(&consent_id).await?;
                Ok(Some(RevolutConsent::new(consent_id, auth_url)))
            } else {
                return Err(RampError::InvalidOrderState(
                    "Expected Revolut provider".to_string(),
                ));
            }
        }
        _ => Ok(None),
    }
}
