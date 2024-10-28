use serde::{Deserialize, Serialize};
use serde_json::to_vec;

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
};

#[derive(Serialize, Deserialize, Debug, candid::CandidType)]
pub struct PaymentResponse {
    pub id: String,
    pub user: UserId,
    pub resource_token: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, candid::CandidType)]
pub struct UserId {
    pub id: String,
}

pub fn parse_payment_response(response_body: Vec<u8>) -> Result<PaymentResponse> {
    let str_body = String::from_utf8(response_body).map_err(|_| SystemError::Utf8Error)?;
    ic_cdk::println!("[parse_payment_response] str_body = {}", str_body);

    if let Ok(error_response) = serde_json::from_str::<super::ErrorResponse>(&str_body) {
        return Err(SystemError::ParseError(error_response.error))?;
    }

    let response: PaymentResponse =
        serde_json::from_str(&str_body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    Ok(response)
}

#[derive(Serialize)]
struct ProxyRequest {
    kid: String,
    #[serde(rename = "privateKeyPem")]
    private_key: String,
    #[serde(rename = "apiUrl")]
    api_url: String,
    path: String,
    body: serde_json::Value,
    #[serde(rename = "idempotencyKey")]
    idempotency_key: String,
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Serialize)]
struct CreatePaymentRequest {
    amount_in_minor: u64,
    currency: String,
    payment_method: PaymentMethod,
    user: PaymentUser,
}

#[derive(Serialize)]
struct PaymentMethod {
    r#type: String,
    provider_selection: ProviderSelection,
    beneficiary: Beneficiary,
}

#[derive(Serialize)]
struct ProviderSelection {
    r#type: String,
    scheme_selection: SchemeSelection,
}

#[derive(Serialize)]
struct SchemeSelection {
    r#type: String,
    allow_remitter_fee: bool,
}

#[derive(Serialize)]
struct Beneficiary {
    r#type: String,
    account_holder_name: String,
    account_identifier: AccountIdentifier,
    reference: String,
}

#[derive(Serialize)]
struct AccountIdentifier {
    r#type: String,
    sort_code: String,
    account_number: String,
}

#[derive(Serialize)]
struct PaymentUser {
    name: String,
    email: String,
}

pub async fn create_payment(
    amount: u64,
    currency: &str,
    account_holder_name: &str,
    sort_code: &str,
    account_number: &str,
    user_name: &str,
    user_email: &str,
) -> Result<PaymentResponse> {
    let access_token = super::auth::get_access_token().await?;
    let (host_url, proxy_url, kid, private_key) = read_state(|s| {
        (
            s.truelayer.host_url.clone(),
            s.truelayer.proxy_url.clone(),
            s.truelayer.kid.clone(),
            s.truelayer.private_key.clone(),
        )
    });

    let private_key = String::from_utf8(private_key).map_err(|_| SystemError::Utf8Error)?;
    let payload = CreatePaymentRequest {
        amount_in_minor: amount,
        currency: currency.to_string(),
        payment_method: PaymentMethod {
            r#type: "bank_transfer".to_string(),
            provider_selection: ProviderSelection {
                r#type: "user_selected".to_string(), // change to preselected and add providerId
                scheme_selection: SchemeSelection {
                    r#type: "instant_only".to_string(),
                    allow_remitter_fee: false,
                },
            },
            beneficiary: Beneficiary {
                r#type: "external_account".to_string(),
                account_holder_name: account_holder_name.to_string(),
                account_identifier: AccountIdentifier {
                    r#type: "sort_code_account_number".to_string(),
                    sort_code: sort_code.to_string(),
                    account_number: account_number.to_string(),
                },
                reference: "Test payment".to_string(),
            },
        },
        user: PaymentUser {
            name: user_name.to_string(),
            email: user_email.to_string(),
        },
    };
    let body =
        serde_json::to_value(&payload).map_err(|e| SystemError::ParseError(e.to_string()))?;

    let proxy_request_body = ProxyRequest {
        kid,
        private_key,
        api_url: format!("https://api.{}", host_url),
        path: "/payments".to_string(),
        body,
        idempotency_key: format!("payment-key-{}-{}", user_email, ic_cdk::api::time()),
        access_token,
    };
    let proxy_body =
        to_vec(&proxy_request_body).map_err(|e| SystemError::ParseError(e.to_string()))?;

    let headers = vec![HttpHeader {
        name: "Content-Type".to_string(),
        value: "application/json".to_string(),
    }];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/generate-signature", proxy_url),
        method: HttpMethod::POST,
        body: Some(proxy_body),
        max_response_bytes: Some(8192),
        transform: None,
        headers,
    };

    let cycles: u128 = 21_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => parse_payment_response(response.body),
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}
