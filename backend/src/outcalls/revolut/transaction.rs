use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse,
};
use num_traits::cast::ToPrimitive;
use serde::Deserialize;

use super::auth::get_revolut_access_token;
use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
};

#[derive(Deserialize, Debug)]
pub struct PaymentDetailsResponse {
    pub data: PaymentData,
}

#[derive(Deserialize, Debug)]
pub struct PaymentData {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Initiation")]
    pub initiation: InitiationDetails,
}

#[derive(Deserialize, Debug)]
pub struct InitiationDetails {
    #[serde(rename = "InstructedAmount")]
    pub instructed_amount: AmountDetails,
    #[serde(rename = "DebtorAccount")]
    pub debtor_account: Option<AccountDetails>,
    #[serde(rename = "CreditorAccount")]
    pub creditor_account: AccountDetails,
}

#[derive(Deserialize, Debug)]
pub struct AmountDetails {
    #[serde(rename = "Amount")]
    pub amount: String,
    #[serde(rename = "Currency")]
    pub currency: String,
}

#[derive(Deserialize, Debug)]
pub struct AccountDetails {
    #[serde(rename = "SchemeName")]
    pub scheme_name: String,
    #[serde(rename = "Identification")]
    pub identification: String,
    #[serde(rename = "Name")]
    pub name: Option<String>,
}

pub async fn fetch_revolut_payment_details(
    domestic_payment_id: &str,
) -> Result<PaymentDetailsResponse> {
    let access_token = get_revolut_access_token().await?;
    let request_headers = vec![
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
        HttpHeader {
            name: "x-fapi-financial-id".to_string(),
            value: "001580000103UAvAAM".to_string(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let api_url = read_state(|s| s.revolut.api_url.clone());
    let request = CanisterHttpRequestArgument {
        url: format!("{}/domestic-payments/{}", api_url, domestic_payment_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(4096),
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    let (response,): (HttpResponse,) = match http_request(request, cycles).await {
        Ok(response) => response,
        Err((code, message)) => {
            return Err(SystemError::HttpRequestError(code as u64, message).into())
        }
    };

    if response.status.0 != 200u64.into() {
        return Err(SystemError::HttpRequestError(
            response.status.0.to_u64().unwrap_or_default(),
            String::from_utf8_lossy(&response.body).to_string(),
        )
        .into());
    }

    let response_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
    ic_cdk::println!(
        "[get_revolut_transactions] Response body: {}",
        response_body
    );

    let payment_details: PaymentDetailsResponse =
        serde_json::from_str(&response_body).map_err(|e| SystemError::ParseError(e.to_string()))?;

    Ok(payment_details)
}
