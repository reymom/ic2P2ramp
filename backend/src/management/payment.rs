use std::time::Duration;

use candid::Principal;
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};

use crate::{
    evm::{transaction, vault::Ic2P2ramp},
    icp::vault::Ic2P2ramp as ICPRamp,
    model::{
        errors::{RampError, Result},
        state::{self, storage},
        types::{
            order::{Order, OrderState},
            PaymentProvider, PaymentProviderType,
        },
    },
    outcalls::revolut,
};

pub async fn handle_evm_payment_completion(
    order_id: u64,
    chain_id: u64,
    gas: Option<u32>,
) -> Result<()> {
    let tx_hash = Ic2P2ramp::release_funds(order_id, chain_id, gas).await?;
    transaction::spawn_transaction_checker(
        tx_hash,
        chain_id,
        60,
        Duration::from_secs(4),
        move || {
            // Update order state to completed
            match super::order::set_order_completed(order_id) {
                Ok(_) => {
                    ic_cdk::println!("[verify_transaction] order {:?} completed", order_id)
                }
                Err(e) => ic_cdk::trap(format!("could not complete order: {:?}", e).as_str()),
            }
        },
    );
    Ok(())
}

pub async fn handle_icp_payment_completion(
    order_id: u64,
    ledger_principal: &Principal,
) -> Result<()> {
    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };

    let onramper_principal = Principal::from_text(&order.onramper_address.address).unwrap();

    let amount = NumTokens::from(order.base.crypto.amount);
    let fee = state::get_fee(ledger_principal)?;

    let to_account = Account {
        owner: onramper_principal,
        subaccount: None,
    };
    ICPRamp::transfer(
        *ledger_principal,
        to_account,
        amount - order.base.crypto.fee,
        Some(fee),
    )
    .await?;

    super::order::set_order_completed(order_id)
}

pub async fn get_revolut_consent(
    order: &Order,
    onramper_provider: &PaymentProvider,
) -> Result<(Option<String>, Option<String>)> {
    let mut revolut_consent_id = None;
    let consent_url = match &onramper_provider {
        PaymentProvider::Revolut {
            scheme: onramper_scheme,
            id: onramper_id,
            ..
        } => {
            let offramper_provider = order
                .offramper_providers
                .get(&PaymentProviderType::Revolut)
                .ok_or_else(|| RampError::ProviderNotInUser(PaymentProviderType::Revolut))?;

            if let PaymentProvider::Revolut {
                scheme: offramper_scheme,
                id: offramper_id,
                name: offramper_name,
            } = offramper_provider
            {
                let consent_id = revolut::consent::create_account_access_consent(
                    &order.fiat_amount.to_string(),
                    &order.currency_symbol,
                    onramper_scheme,
                    onramper_id,
                    &offramper_scheme,
                    &offramper_id,
                    &offramper_name
                        .clone()
                        .ok_or_else(|| RampError::InvalidOfframperProvider)?,
                )
                .await?;
                revolut_consent_id = Some(consent_id.clone());
                Some(revolut::authorize::get_authorization_url(&consent_id).await?)
            } else {
                return Err(RampError::InvalidOrderState(
                    "Expected Revolut provider".to_string(),
                ));
            }
        }
        _ => None,
    };

    Ok((revolut_consent_id, consent_url))
}
