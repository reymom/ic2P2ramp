// applicationName: icRamp
// clientId: sandbox-icramp-8c54b8
// clientSecret: 78d0c1b4-21e1-441d-b94b-052f255e0d5e

pub async fn verify_truelayer_payment(payment_id: &str) -> Result<PaymentStatus> {
    let api_url = format!("https://api.truelayer.com/payments/{}", payment_id);
    let auth_token = "YOUR_AUTH_TOKEN";

    let request_headers = vec![
        HttpHeader {
            name: "Authorization".to_string(),
            value: format!("Bearer {}", auth_token),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: api_url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(4096),
        transform: None,
        headers: request_headers,
    };

    match http_request(request, 10_000_000_000).await {
        Ok((response,)) => {
            let payment_status: PaymentStatus = serde_json::from_slice(&response.body)?;
            Ok(payment_status)
        }
        Err((code, msg)) => Err(SystemError::HttpRequestError(code as u64, msg).into()),
    }
}
