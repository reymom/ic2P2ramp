use serde::Serialize;

use crate::{
    errors::{Result, SystemError},
    model::memory::heap::read_state,
    outcalls::revolut::jws,
};

#[derive(Serialize, Debug)]
struct JwtClaims {
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: String,
    claims: Claims,
}

#[derive(Serialize, Debug)]
struct Claims {
    id_token: IdTokenClaims,
}

#[derive(Serialize, Debug)]
struct IdTokenClaims {
    openbanking_intent_id: OpenBankingIntentId, // ConsentId
}

#[derive(Serialize, Debug)]
struct OpenBankingIntentId {
    value: String,
}

pub async fn get_authorization_url(consent_id: &str) -> Result<String> {
    let (api_url, client_id, proxy_url, kid) = read_state(|s| {
        (
            s.revolut.api_url.clone(),
            s.revolut.client_id.clone(),
            s.revolut.proxy_url.clone(),
            s.revolut.kid.clone(),
        )
    });

    let jws_header = jws::JWSHeader::new_simple(&kid);

    let redirect_uri = format!("{}/revolut/exchange", proxy_url);
    let jwt_claims = JwtClaims {
        response_type: "code id_token".to_string(),
        client_id: client_id.to_string(),
        redirect_uri: redirect_uri.clone(),
        scope: "payments".to_string(),
        claims: Claims {
            id_token: IdTokenClaims {
                openbanking_intent_id: OpenBankingIntentId {
                    value: consent_id.to_string(),
                },
            },
        },
    };

    ic_cdk::println!("jwt_claims = {:?}", jwt_claims);
    let jwt_claims_str =
        serde_json::to_string(&jwt_claims).map_err(|e| -> SystemError { e.into() })?;

    ic_cdk::println!("jwt_claims_str = {:?}", jwt_claims_str);
    let jwt = jws::create_jws_signature(&jwt_claims_str, &jws_header).await?;
    let url = format!(
        "{}/ui/index.html?response_type=code%20id_token&scope=payments&redirect_uri={}&client_id={}&request={}&state={}",
        api_url,
        redirect_uri,
        client_id,
        jwt,
        consent_id
    );

    Ok(url)
}
