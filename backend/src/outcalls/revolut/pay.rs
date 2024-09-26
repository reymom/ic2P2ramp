use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext, TransformFunc,
};
use serde::{Deserialize, Serialize};

use crate::{errors::SystemError, model::memory::heap::read_state, Result};

use super::jws;

#[derive(Serialize, Deserialize)]
struct PaymentResponse {
    #[serde(rename = "Data")]
    data: PaymentData,
}

#[derive(Serialize, Deserialize)]
struct PaymentData {
    #[serde(rename = "DomesticPaymentId")]
    domestic_payment_id: String,
}

pub async fn initiate_domestic_payment(
    consent_id: &str,
    access_token: &str,
    amount: &str,
    currency: &str,
    onramper_scheme: &str,
    onramper_id: &str,
    offramper_scheme: &str,
    offramper_id: &str,
    offramper_name: &str,
) -> Result<String> {
    let jws_payload = format!(
        r#"
        {{
        "Data": {{
            "ConsentId": "{}",
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
        consent_id,
        amount,
        currency,
        onramper_scheme,
        onramper_id,
        offramper_scheme,
        offramper_id,
        offramper_name
    );

    let (api_url, kid, tan) = read_state(|s| {
        (
            s.revolut.api_url.clone(),
            s.revolut.kid.clone(),
            s.revolut.tan.clone(),
        )
    });
    let jws_header = jws::JWSHeader::new(&kid, &tan);
    let jws_signature = jws::create_jws_signature(&jws_payload, &jws_header).await?;

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
            value: ic_cdk::api::time().to_string(),
        },
        HttpHeader {
            name: "x-jws-signature".to_string(),
            value: jws_signature,
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/domestic-payments", api_url),
        method: HttpMethod::POST,
        body: Some(jws_payload.as_bytes().to_vec()),
        max_response_bytes: Some(4096),
        transform: Some(TransformContext {
            function: TransformFunc(candid::Func {
                principal: ic_cdk::api::id(),
                method: "transform_revolut_payment_response".to_string(),
            }),
            context: vec![],
        }),
        headers: request_headers,
    };

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| SystemError::Utf8Error)?;
            ic_cdk::println!("[initiate_domestic_payment] Response body: {}", str_body);

            let payment_response: PaymentResponse = serde_json::from_str(&str_body)
                .map_err(|e| SystemError::ParseError(e.to_string()))?;
            Ok(payment_response.data.domestic_payment_id)
        }
        Err((r, m)) => Err(SystemError::HttpRequestError(r as u64, m).into()),
    }
}

#[ic_cdk::query]
fn transform_revolut_payment_response(args: TransformArgs) -> HttpResponse {
    let mut response = args.response;

    if let Ok(data_response) = serde_json::from_slice::<PaymentResponse>(&response.body) {
        if let Ok(payment_id_json) = serde_json::to_vec(&data_response.data) {
            response.body = payment_id_json;
        }
    }

    response
}
