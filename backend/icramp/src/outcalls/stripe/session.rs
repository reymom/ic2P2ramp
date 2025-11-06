use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, http_request,
};

use crate::{
    model::{
        errors::{Result, SystemError},
        memory::heap::read_state,
        types::payment::stripe::{CheckoutSession, pick_platform},
    },
    outcalls::stripe::{auth_headers, pct_encode},
};

// Create Checkout Session (destination charge)
pub async fn create_checkout_session_for_order(
    order_id: u64,
    offramper_acct: &str,
    amount_minor: u64,
    currency: &str, // e.g. "EUR"
    platform_label: Option<String>,
    success_url: String,
    cancel_url: String,
    payer_email: String,
) -> Result<(String, String)> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    // NOTE: currency must be lowercase for Stripe API
    let cur = currency.to_lowercase();
    let email_q = format!(
        "&customer_email={ce}&payment_intent_data[receipt_email]={re}",
        ce = pct_encode(&payer_email),
        re = pct_encode(&payer_email),
    );

    let body = format!(
        "mode=payment\
            &success_url={success}\
            &cancel_url={cancel}\
            &line_items[0][quantity]=1\
            &line_items[0][price_data][currency]={cur}\
            &line_items[0][price_data][unit_amount]={amt}\
            &line_items[0][price_data][product_data][name]=Order%20{oid}\
            &payment_intent_data[transfer_data][destination]={dest}\
            {email_q}",
        success = pct_encode(&success_url),
        cancel = pct_encode(&cancel_url),
        cur = cur,
        amt = amount_minor,
        oid = order_id,
        dest = offramper_acct,
        email_q = email_q
    );

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/checkout/sessions", proxy_url),
        method: HttpMethod::POST,
        body: Some(body.into_bytes()),
        max_response_bytes: Some(20_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-create-{}-{}", order_id, ic_cdk::api::time()),
        ),
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;

    let v: serde_json::Value =
        serde_json::from_slice(&resp.body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    if let Some(err) = v.get("error") {
        let msg = err
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown");
        let param = err.get("param").and_then(|x| x.as_str()).unwrap_or("-");
        let req_url = err
            .get("request_log_url")
            .and_then(|x| x.as_str())
            .unwrap_or("-");
        return Err(SystemError::InvalidInput(format!(
            "Stripe error: {msg} [param={param}] (log={req_url})"
        ))
        .into());
    }

    let id = v["id"].as_str().unwrap_or_default().to_string();
    let url = v["url"].as_str().unwrap_or_default().to_string();
    if id.is_empty() || url.is_empty() {
        return Err(SystemError::ParseError("missing session id/url".into()).into());
    }
    Ok((id, url))
}

pub async fn retrieve_session(
    session_id: &str,
    expand_payment_intent: bool,
    platform_label: Option<String>,
) -> Result<CheckoutSession> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());
    let url = if expand_payment_intent {
        format!(
            "{}/v1/checkout/sessions/{}?expand[]=payment_intent&expand[]=payment_intent.charges",
            proxy_url, session_id
        )
    } else {
        format!("{}/v1/checkout/sessions/{}", proxy_url, session_id)
    };

    let req = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(40_000),
        transform: None,
        headers: vec![
            HttpHeader {
                name: "Authorization".into(),
                value: format!("Bearer {}", p.secret_key),
            },
            HttpHeader {
                name: "x-forwarded-host".into(),
                value: p.api_url.clone(),
            },
            HttpHeader {
                name: "idempotency-key".into(),
                value: format!("stripe-retrieve-{}", session_id),
            },
        ],
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let s: CheckoutSession =
        serde_json::from_slice(&resp.body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    Ok(s)
}
