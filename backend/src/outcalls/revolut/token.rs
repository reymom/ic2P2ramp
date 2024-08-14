use num_traits::cast::ToPrimitive;
use std::time::Duration;

use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use crate::{
    evm::helpers,
    management::order,
    model::{
        errors::{RampError, Result},
        state::{read_state, storage},
        types::{order::OrderState, PaymentProvider},
    },
    outcalls::revolut::pay,
};

pub async fn get_revolut_access_token(consent_id: String) -> Result<String> {
    let proxy_url = read_state(|s| s.revolut.proxy_url.clone());
    let url = format!(
        "{}/revolut/payment_token?consent_id={}",
        proxy_url, consent_id
    );

    let response = ic_cdk::api::management_canister::http_request::http_request(
        CanisterHttpRequestArgument {
            url,
            method: HttpMethod::GET,
            body: None,
            max_response_bytes: Some(4096),
            transform: None,
            headers: vec![HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            }],
        },
        10_000_000_000,
    )
    .await
    .map_err(|(code, msg)| RampError::HttpRequestError(code as u64, msg))?;

    let response_str = String::from_utf8(response.0.body).map_err(|_| RampError::Utf8Error)?;

    if response.0.status == 404_u32 {
        return Err(RampError::HttpRequestError(
            404,
            "No token found for the given ConsentId".to_string(),
        ));
    }

    if response.0.status != 200_u32 {
        return Err(RampError::HttpRequestError(
            response.0.status.0.to_u64().unwrap_or_default(),
            response_str.clone(),
        ));
    }

    let token_response: serde_json::Value =
        serde_json::from_str(&response_str).map_err(|e| RampError::ParseError(e.to_string()))?;

    if let Some(access_token) = token_response.get("access_token").and_then(|v| v.as_str()) {
        Ok(access_token.to_string())
    } else {
        Err(RampError::MissingAccessToken)
    }
}

pub async fn wait_for_revolut_access_token(
    order_id: u64,
    max_attempts: u32,
    interval_seconds: u64,
) -> Result<String> {
    let order_state = storage::get_order(&order_id)?;
    let order = match order_state {
        OrderState::Locked(locked_order) => locked_order,
        _ => return Err(RampError::InvalidOrderState(order_state.to_string())),
    };
    let (
        consent_id,
        amount,
        currency,
        debtor_scheme,
        debtor_id,
        creditor_scheme,
        creditor_id,
        creditor_name,
    ) = {
        let PaymentProvider::Revolut { scheme, id, name } = order.onramper_provider else {
            return Err(RampError::InvalidOnramperProvider);
        };
        let name = match name.clone() {
            Some(name) => name,
            None => return Err(RampError::InvalidOnramperProvider),
        };
        let consent_id = match order.consent_id.clone() {
            Some(consent_id) => consent_id,
            None => return Err(RampError::InvalidOnramperProvider),
        };

        (
            consent_id.clone(),
            order.base.fiat_amount.to_string(),
            order.base.currency_symbol,
            scheme.clone(),
            id.clone(),
            scheme.clone(),
            id.clone(),
            name.clone(),
        )
    };

    for attempt in 0..max_attempts {
        match get_revolut_access_token(consent_id.clone()).await {
            Ok(access_token) => {
                ic_cdk::println!("[wait_for_access_token] Access token retrieved.");
                let payment_id = pay::initiate_domestic_payment(
                    &consent_id,
                    &access_token,
                    &amount,
                    &currency,
                    &debtor_scheme,
                    &debtor_id,
                    &creditor_scheme,
                    &creditor_id,
                    &creditor_name,
                )
                .await?;

                order::set_payment_id(order_id, payment_id.clone())?;

                // Automatically verify the transaction after setting the payment ID
                ic_cdk::println!("[wait_for_access_token] Verifying transaction...");
                match crate::verify_transaction(order_id, payment_id.clone(), None).await {
                    Ok(_) => ic_cdk::println!(
                        "[wait_for_access_token] Transaction verified successfully."
                    ),
                    Err(e) => ic_cdk::println!(
                        "[wait_for_access_token] Failed to verify transaction: {:?}",
                        e
                    ),
                }

                return Ok(payment_id);
            }
            Err(_) => {
                if attempt + 1 >= max_attempts {
                    return Err(RampError::TransactionTimeout);
                }
                ic_cdk::println!(
                    "[wait_for_access_token] Access token not yet available. Attempt: {}",
                    attempt
                );
            }
        }
        ic_cdk::println!(
            "[wait_for_access_token] Waiting for {} seconds before next attempt.",
            interval_seconds
        );
        helpers::delay(Duration::from_secs(interval_seconds)).await;
    }
    Err(RampError::TransactionTimeout)
}
