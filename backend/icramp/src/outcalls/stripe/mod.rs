use ic_cdk::api::management_canister::http_request::HttpHeader;

pub mod account;
pub mod session;

pub(self) fn pct_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => {
                use core::fmt::Write as _;
                let _ = write!(&mut out, "%{:02X}", b);
            }
        }
    }
    out
}

pub(self) fn auth_headers(sk: &str, api_url: &str, idem: &str) -> Vec<HttpHeader> {
    vec![
        HttpHeader {
            name: "Content-Type".into(),
            value: "application/x-www-form-urlencoded".into(),
        },
        HttpHeader {
            name: "Authorization".into(),
            value: format!("Bearer {}", sk),
        },
        HttpHeader {
            name: "x-forwarded-host".into(),
            value: api_url.into(),
        },
        HttpHeader {
            name: "idempotency-key".into(),
            value: idem.into(),
        },
    ]
}
