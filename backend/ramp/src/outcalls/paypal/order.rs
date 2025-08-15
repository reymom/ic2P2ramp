use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct PayPalOrderDetails {
    id: String,
    pub status: String,
    pub purchase_units: Vec<PurchaseUnit>,
    pub payer: Payer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PurchaseUnit {
    pub amount: Amount,
    pub payee: Payee,
    pub payments: Payments,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Payments {
    pub captures: Vec<Capture>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Capture {
    id: String,
    status: String,
    pub amount: Amount,
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
pub struct Payer {
    pub email_address: String,
    payer_id: String,
}

pub async fn fetch_paypal_order(access_token: &str, order_id: &str) -> Result<PayPalOrderDetails> {
    let (api_url, proxy_url) = read_state(|s| (s.paypal.api_url.clone(), s.proxy_url.clone()));

    let request_headers = vec![
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", access_token),
        },
        HttpHeader {
            name: "x-forwarded-host".to_string(),
            value: api_url,
        },
        HttpHeader {
            name: "idempotency-key".to_string(),
            value: format!("order-key-{}-{}", order_id, ic_cdk::api::time()).to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/v2/checkout/orders/{}", proxy_url, order_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(5096), // content-length is 2630 bytes
        transform: None,
        headers: request_headers,
    };

    let cycles = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
            ic_cdk::println!("[fetch_paypal_order] str_body = {:?}", str_body);

            let order_details: PayPalOrderDetails = serde_json::from_str(&str_body)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;
            ic_cdk::println!("[fetch_paypal_order] order_details = {:?}", order_details);

            Ok(order_details)
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
