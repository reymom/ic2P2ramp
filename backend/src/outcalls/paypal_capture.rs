use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use serde::{Deserialize, Serialize};

use crate::errors::{RampError, Result};

#[derive(Serialize, Deserialize, Debug)]
pub struct PayPalCaptureDetails {
    id: String,
    pub status: String,
    pub amount: Amount,
    pub payee: Payee,
    pub supplementary_data: SupplementaryData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Amount {
    pub currency_code: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payee {
    pub email_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SupplementaryData {
    pub related_ids: RelatedIds,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RelatedIds {
    pub order_id: String,
}

pub async fn fetch_paypal_capture_details(
    access_token: &str,
    transaction_id: &str,
    cycles: u128,
) -> Result<PayPalCaptureDetails> {
    let url = format!(
        "https://api-m.sandbox.paypal.com/v2/payments/captures/{}",
        transaction_id
    );

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: request_headers,
    };

    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            ic_cdk::println!("[fetch_paypal_capture_details] str_body = {:?}", str_body);

            let capture_details: PayPalCaptureDetails = serde_json::from_str(&str_body)
                .map_err(|e| RampError::ParseError(e.to_string()))?;
            ic_cdk::println!(
                "[fetch_paypal_capture_details] capture_details = {:?}",
                capture_details
            );

            Ok(capture_details)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
