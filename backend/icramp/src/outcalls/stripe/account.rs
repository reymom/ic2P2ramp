use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpMethod, http_request,
};
use serde::Deserialize;

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
    types::{
        payment::stripe::pick_platform,
        stripe::{StripeAccountInfo, StripeCapabilities},
    },
};
use crate::outcalls::stripe::{auth_headers, pct_encode};

pub async fn get_account_raw(account_id: &str, platform_label: Option<String>) -> Result<String> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/accounts/{}", proxy_url, account_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(20_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-acct-{}", ic_cdk::api::time()),
        ),
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let body = String::from_utf8(resp.body).map_err(|_| SystemError::Utf8Error)?;
    Ok(body)
}

pub async fn get_account_info(
    account_id: &str,
    platform_label: Option<String>,
) -> Result<StripeAccountInfo> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/accounts/{}", proxy_url, account_id),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(40_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-acct-{}", ic_cdk::api::time()),
        ),
    };

    let (resp,) = http_request(req, 21_000_000_000u128)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;

    #[derive(Deserialize)]
    struct Raw {
        id: String,
        country: String,
        default_currency: Option<String>,
        payouts_enabled: bool,
        charges_enabled: bool,
        details_submitted: bool,
        capabilities: StripeCapabilities,
    }

    let raw: Raw = serde_json::from_slice(&resp.body).map_err(SystemError::from)?;

    Ok(StripeAccountInfo {
        id: raw.id,
        country: raw.country,
        default_currency: raw.default_currency,
        payouts_enabled: raw.payouts_enabled,
        charges_enabled: raw.charges_enabled,
        details_submitted: raw.details_submitted,
        capabilities: raw.capabilities,
    })
}

pub async fn create_express_account(
    email: &str,
    country: &str, // e.g. "ES" or "US"
    platform_label: Option<String>,
) -> Result<String> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let body = format!(
        "type=express&country={}&email={}&capabilities[card_payments][requested]=true&capabilities[transfers][requested]=true",
        pct_encode(country),
        pct_encode(email),
    );

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/accounts", proxy_url),
        method: HttpMethod::POST,
        body: Some(body.into_bytes()),
        max_response_bytes: Some(20_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-acc-create-{}", ic_cdk::api::time()),
        ),
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let v: serde_json::Value =
        serde_json::from_slice(&resp.body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    let id = v["id"].as_str().unwrap_or_default().to_string();
    if id.is_empty() {
        return Err(SystemError::ParseError("missing account id".into()).into());
    }
    Ok(id)
}

pub async fn create_account_link(
    account_id: &str,  // acct_...
    refresh_url: &str, // where to send them if they abandon
    return_url: &str,  // where to send them after completion
    platform_label: Option<String>,
) -> Result<String> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let body = format!(
        "type=account_onboarding&account={}&refresh_url={}&return_url={}",
        pct_encode(account_id),
        pct_encode(refresh_url),
        pct_encode(return_url),
    );

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/account_links", proxy_url),
        method: HttpMethod::POST,
        body: Some(body.into_bytes()),
        max_response_bytes: Some(20_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-acc-link-{}", ic_cdk::api::time()),
        ),
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let v: serde_json::Value =
        serde_json::from_slice(&resp.body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    let url = v["url"].as_str().unwrap_or_default().to_string();
    if url.is_empty() {
        return Err(SystemError::ParseError("missing account link url".into()).into());
    }
    Ok(url)
}

pub async fn create_login_link(
    account_id: &str, // acct_...
    platform_label: Option<String>,
) -> Result<String> {
    let p = pick_platform(platform_label)?;
    let proxy_url = read_state(|s| s.proxy_url.clone());

    let req = CanisterHttpRequestArgument {
        url: format!("{}/v1/accounts/{}/login_links", proxy_url, account_id),
        method: HttpMethod::POST,
        body: Some(Vec::new()), // empty form body
        max_response_bytes: Some(20_000),
        transform: None,
        headers: auth_headers(
            &p.secret_key,
            &p.api_url,
            &format!("stripe-login-link-{}", ic_cdk::api::time()),
        ),
    };

    let cycles: u128 = 21_000_000_000;
    let (resp,) = http_request(req, cycles)
        .await
        .map_err(|(r, m)| SystemError::HttpRequestError(r as u64, m))?;
    let v: serde_json::Value =
        serde_json::from_slice(&resp.body).map_err(|e| SystemError::ParseError(e.to_string()))?;
    let url = v["url"].as_str().unwrap_or_default().to_string();
    if url.is_empty() {
        return Err(SystemError::ParseError("missing login link url".into()).into());
    }
    Ok(url)
}
