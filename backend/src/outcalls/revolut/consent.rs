use crate::{
    errors::{RampError, Result},
    state::read_state,
};
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AccountAccessConsentResponse {
    ConsentId: String,
}

pub async fn create_account_access_consent() -> Result<String> {
    let access_token = super::auth::get_revolut_access_token().await?;
    let api_url = read_state(|s| s.revolut.api_url.clone());

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
        HttpHeader {
            name: "x-idempotency-key".to_string(),
            value: "random_key".to_string(),
        },
        HttpHeader {
            name: "x-jws-signature".to_string(),
            value: "<insert JWS>".to_string(),
        },
    ];

    let request_body = r#"
    {
        "Data": {
            "Initiation": {
            "InstructionIdentification": "ID412",
            "EndToEndIdentification": "E2E123",
            "InstructedAmount": {
                "Amount": "55.00",
                "Currency": "GBP"
            },
            "CreditorAccount": {
                "SchemeName": "UK.OBIE.SortCodeAccountNumber",
                "Identification": "11223321325698",
                "Name": "Receiver Co."
            },
            "RemittanceInformation": {
                "Unstructured": "Shipment fee"
            }
            }
        },
        "Risk": {}
    }
    "#;

    let request = CanisterHttpRequestArgument {
        url: format!("{}/domestic-payment-consents", api_url),
        method: HttpMethod::POST,
        body: Some(request_body.as_bytes().to_vec()),
        max_response_bytes: Some(4096), // Check the response locally
        transform: None,
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            ic_cdk::println!(
                "[create_account_access_consent] Response body: {}",
                str_body
            );

            let consent_response: AccountAccessConsentResponse = serde_json::from_str(&str_body)
                .map_err(|e| RampError::ParseError(e.to_string()))?;
            Ok(consent_response.ConsentId)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
