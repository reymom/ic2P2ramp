use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{
    errors::{RampError, Result},
    state::read_state,
    types::revolut,
};

#[derive(Serialize)]
struct Claims {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
    claims: IdTokenClaims,
}

#[derive(Serialize)]
struct IdTokenClaims {
    openbanking_intent_id: OpenBankingIntentId, // ConsentId
}

#[derive(Serialize)]
struct OpenBankingIntentId {
    value: String,
}

fn generate_jwt(
    consent_id: &str,
    client_id: &str,
    redirect_uri: &str,
    kid: &str,
    private_key: &[u8],
) -> Result<String> {
    let claims = Claims {
        response_type: "code id_token".to_string(),
        client_id: client_id.to_string(),
        redirect_uri: redirect_uri.to_string(),
        scope: "payments".to_string(),
        claims: IdTokenClaims {
            openbanking_intent_id: OpenBankingIntentId {
                value: consent_id.to_string(),
            },
        },
    };

    let mut header = Header::new(jsonwebtoken::Algorithm::PS256);
    header.kid = Some(kid.to_string());

    encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(private_key).map_err(|_| RampError::JwtError),
    )
}

pub fn get_authorization_url(jwt: &str, client_id: &str, redirect_uri: &str) -> String {
    format!(
        "https://sandbox-oba.revolut.com/ui/index.html?response_type=code%20id_token&scope=accounts&redirect_uri={}&client_id={}&request={}&state=example_state",
        urlencoding::encode(redirect_uri),
        urlencoding::encode(client_id),
        urlencoding::encode(jwt)
    )
}

#[derive(Serialize, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expires_at: u64,
    id_token: String,
}

pub async fn exchange_authorization_code(code: &str) -> Result<String> {
    let api_url = read_state(|s| s.revolut.proxy_url.clone());

    let request_headers = vec![HttpHeader {
        name: "Content-Type".to_string(),
        value: "application/json".to_string(),
    }];

    let request = CanisterHttpRequestArgument {
        url: format!("{}/revolut/consent/{}", api_url, code),
        method: HttpMethod::POST,
        body: None,
        max_response_bytes: Some(1024), // content-length is 576 bytes
        transform: None,
        headers: request_headers,
    };

    ic_cdk::println!("[get_revolut_access_token] request = {:?}", request);

    let cycles: u128 = 10_000_000_000;
    match http_request(request, cycles).await {
        Ok((response,)) => {
            let str_body = String::from_utf8(response.body).map_err(|_| RampError::Utf8Error)?;
            ic_cdk::println!("[get_revolut_access_token] Response body: {}", str_body);

            let access_token_response: AccessTokenResponse = serde_json::from_str(&str_body)
                .map_err(|e| RampError::ParseError(e.to_string()))?;

            // Store the token and its expiration time in the state
            let expiration_time = access_token_response.expires_at;
            ic_cdk::println!(
                "[get_revolut_access_token] New token expiration time: {}",
                expiration_time
            );
            revolut::set_revolut_token(access_token_response.access_token.clone(), expiration_time);

            Ok(access_token_response.access_token)
        }
        Err((r, m)) => Err(RampError::HttpRequestError(r as u64, m)),
    }
}
