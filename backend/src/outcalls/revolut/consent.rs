use serde::{Deserialize, Serialize};

use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
};

use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext, TransformFunc,
};

use super::jws;

#[derive(Serialize, Deserialize, Debug)]
struct ConsentIdResponse {
    #[serde(rename = "ConsentId")]
    consent_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DataResponse {
    #[serde(rename = "Data")]
    data: ConsentIdResponse,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Errors")]
    errors: Vec<ErrorDetail>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorDetail {
    #[serde(rename = "ErrorCode")]
    error_code: String,
    #[serde(rename = "Message")]
    message: String,
}

pub async fn create_account_access_consent(
    amount: &str,
    currency: &str,
    debtor_scheme: &str,
    debtor_id: &str,
    creditor_scheme: &str,
    creditor_id: &str,
    creditor_name: &str,
) -> Result<String> {
    let access_token = super::auth::get_revolut_access_token().await?;
    let (api_url, kid, tan) = read_state(|s| {
        (
            s.revolut.api_url.clone(),
            s.revolut.kid.clone(),
            s.revolut.tan.clone(),
        )
    });

    let jws_header = jws::JWSHeader::new(&kid, &tan);
    let jws_payload = format!(
        r#"
        {{
        "Data": {{
            "Initiation": {{
                "InstructionIdentification": "ID412",
                "EndToEndIdentification": "E2E123",
                "InstructedAmount": {{
                    "Amount": "{}",
                    "Currency": "{}"
                }},
                "DebtorAccount": {{
                    "SchemeName": "{}",
                    "Identification": "{}"
                }},
                "CreditorAccount": {{
                    "SchemeName": "{}",
                    "Identification": "{}",
                    "Name": "{}"
                }}
            }}
        }},
        "Risk": {{
            "PaymentContextCode": "PartyToParty"
        }}
    }}
    "#,
        amount, currency, debtor_scheme, debtor_id, creditor_scheme, creditor_id, creditor_name
    );

    let jws_signature = jws::create_jws_signature(&jws_payload, &jws_header).await?;
    let idempotency_key = ic_cdk::api::time().to_string();
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
            value: idempotency_key,
        },
        HttpHeader {
            name: "x-jws-signature".to_string(),
            value: jws_signature,
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/domestic-payment-consents", api_url),
        method: HttpMethod::POST,
        body: Some(jws_payload.as_bytes().to_vec()),
        max_response_bytes: Some(8192),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform_revolut_consent_response".to_string(),
            }),
            context: vec![],
        }),
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
            ic_cdk::println!(
                "[create_account_access_consent] Response body: {}",
                str_body
            );

            if let Ok(consent_response) = serde_json::from_str::<ConsentIdResponse>(&str_body) {
                Ok(consent_response.consent_id)
            } else if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&str_body) {
                ic_cdk::println!(
                    "[create_account_access_consent] Error response: {:?}",
                    error_response
                );
                Err(SystemError::ParseError(format!(
                    "API Error: {} - {}",
                    error_response.code, error_response.message
                ))
                .into())
            } else {
                Err(SystemError::ParseError("Unknown response format".to_string()).into())
            }
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}

#[ic_cdk::query]
fn transform_revolut_consent_response(args: TransformArgs) -> HttpResponse {
    let mut response = args.response;

    // Extract and retain only the Data.ConsentId field
    if let Ok(data_response) = serde_json::from_slice::<DataResponse>(&response.body) {
        if let Ok(consent_id_json) = serde_json::to_vec(&data_response.data) {
            response.body = consent_id_json;
        }
    }

    response
}
