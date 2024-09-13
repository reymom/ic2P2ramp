use std::time::Duration;

use candid::Principal;
use icrc_ledger_types::icrc1::{account::Account, transfer::NumTokens};
use num_traits::cast::ToPrimitive;

use crate::{
    evm::{transaction, vault::Ic2P2ramp},
    icp::vault::Ic2P2ramp as ICPRamp,
    model::types::{
        evm::gas::{self, MethodGasUsage},
        order::{Order, OrderState},
        PaymentProvider, PaymentProviderType,
    },
    model::{
        errors::{RampError, Result},
        state::{self, storage},
    },
    outcalls::revolut,
};

pub async fn handle_evm_payment_completion(
    order_id: u64,
    chain_id: u64,
    gas: Option<u64>,
) -> Result<String> {
    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };
    let mut action_type = MethodGasUsage::ReleaseNative;
    if let Some(_) = order.base.crypto.token {
        action_type = MethodGasUsage::ReleaseToken
    };
    let tx_hash = Ic2P2ramp::release_funds(order, chain_id, gas).await?;

    transaction::spawn_transaction_checker(
        tx_hash.clone(),
        chain_id,
        60,
        Duration::from_secs(4),
        move |receipt| {
            let gas_used = receipt.gasUsed.0.to_u128().unwrap_or(0);
            let gas_price = receipt.effectiveGasPrice.0.to_u128().unwrap_or(0);
            let block_number = receipt.blockNumber.0.to_u128().unwrap_or(0);

            ic_cdk::println!(
                "[verify_transaction] Gas Used: {}, Gas Price: {}, Block Number: {}",
                gas_used,
                gas_price,
                block_number
            );

            if !(gas_used == 0 || gas_price == 0) {
                let _ = gas::register_gas_usage(
                    chain_id,
                    gas_used,
                    gas_price,
                    block_number,
                    &action_type,
                );
            }

            // Update order state to completed
            match super::order::set_order_completed(order_id) {
                Ok(_) => {
                    ic_cdk::println!("[verify_transaction] order {:?} completed", order_id)
                }
                Err(e) => ic_cdk::trap(format!("could not complete order: {:?}", e).as_str()),
            }
        },
    );
    Ok(tx_hash)
}

pub async fn handle_icp_payment_completion(
    order_id: u64,
    ledger_principal: &Principal,
) -> Result<String> {
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
