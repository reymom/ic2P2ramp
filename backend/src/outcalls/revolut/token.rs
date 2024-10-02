use num_traits::cast::ToPrimitive;
use std::time::Duration;

use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use crate::{
    errors::{BlockchainError, OrderError, Result, SystemError},
    management::order,
    model::{
        helpers,
        memory::{heap::read_state, stable},
    },
    outcalls::revolut::pay,
    types::PaymentProvider,
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
    .map_err(|(code, msg)| SystemError::HttpRequestError(code as u64, msg))?;

    let response_str = String::from_utf8(response.0.body).map_err(|_| SystemError::Utf8Error)?;

    if response.0.status == 404_u32 {
        return Err(SystemError::HttpRequestError(
            404,
            "No token found for the given ConsentId".to_string(),
        )
        .into());
    }

    if response.0.status != 200_u32 {
        return Err(SystemError::HttpRequestError(
            response.0.status.0.to_u64().unwrap_or_default(),
            response_str.clone(),
        )
        .into());
    }

    let token_response: serde_json::Value =
        serde_json::from_str(&response_str).map_err(|e| SystemError::ParseError(e.to_string()))?;

    if let Some(access_token) = token_response.get("access_token").and_then(|v| v.as_str()) {
        Ok(access_token.to_string())
    } else {
        Err(OrderError::MissingAccessToken.into())
    }
}

pub async fn wait_for_revolut_access_token(
    order_id: u64,
    session_token: &str,
    max_attempts: u32,
    interval_seconds: u64,
) -> Result<String> {
    let order = stable::orders::get_order(&order_id)?.locked()?;

    let user = stable::users::get_user(&order.onramper.user_id)?;
    user.validate_session(session_token)?;

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
        let PaymentProvider::Revolut { scheme, id, name } = order.onramper.provider else {
            return Err(OrderError::InvalidOnramperProvider.into());
        };
        let name = match name.clone() {
            Some(name) => name,
            None => return Err(OrderError::InvalidOnramperProvider.into()),
        };
        let consent_id = match order.revolut_consent {
            Some(consent_id) => consent_id.id,
            None => return Err(OrderError::InvalidOnramperProvider.into()),
        };

        (
            consent_id.clone(),
            (order.price as f64 / 100.).to_string(),
            order.base.currency,
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
                match crate::verify_transaction(
                    order_id,
                    Some(session_token.to_string()),
                    payment_id.clone(),
                )
                .await
                {
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
                    return Err(BlockchainError::TransactionTimeout.into());
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
    Err(BlockchainError::TransactionTimeout.into())
}
