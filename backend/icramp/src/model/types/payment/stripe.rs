use candid::{CandidType, Deserialize};
use serde::Serialize;

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
};

#[derive(Clone, CandidType, Deserialize)]
pub struct StripePlatformState {
    pub label: String,
    pub api_url: String,
    pub publishable_key: String,
    pub secret_key: String, // used to sign "Authorization: Bearer ..."
}

impl core::fmt::Debug for StripePlatformState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StripePlatformState")
            .field("label", &self.label)
            .field("api_url", &self.api_url)
            .field("publishable_key", &self.publishable_key)
            .field("secret_key", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, CandidType, Deserialize)]
pub struct StripeState {
    pub default_platform: String,
    pub platforms: std::collections::HashMap<String, StripePlatformState>,
}

impl core::fmt::Debug for StripeState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let labels: Vec<&str> = self.platforms.keys().map(|s| s.as_str()).collect();
        f.debug_struct("StripeState")
            .field("default_platform", &self.default_platform)
            .field("platform_labels", &labels)
            .finish()
    }
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StripePlatformConfig {
    pub label: String,           // e.g. "ES" or "US"
    pub api_url: String,         // "https://api.stripe.com"
    pub publishable_key: String, // pk_test_...
    pub secret_key: String,      // sk_test_...  (stored in canister, used in Authorization)
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct StripeConfig {
    pub default_platform: String,
    pub platforms: Vec<StripePlatformConfig>,
}

pub fn stripe_state_from_config(cfg: StripeConfig) -> StripeState {
    let mut map = std::collections::HashMap::new();
    for p in cfg.platforms {
        let s = StripePlatformState {
            label: p.label.clone(),
            api_url: p.api_url,
            publishable_key: p.publishable_key,
            secret_key: p.secret_key,
        };
        map.insert(p.label, s);
    }
    StripeState {
        default_platform: cfg.default_platform,
        platforms: map,
    }
}

pub fn pick_platform(label: Option<String>) -> Result<StripePlatformState> {
    read_state(|s| {
        let lbl = label.unwrap_or_else(|| s.stripe.default_platform.clone());
        s.stripe.platforms.get(&lbl).cloned().ok_or(
            SystemError::InvalidInput(format!("Unknown Stripe platform label: {}", lbl)).into(),
        )
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CustomerDetails {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckoutSession {
    pub id: String,
    pub url: Option<String>,
    pub payment_status: Option<String>,
    pub payment_intent: Option<serde_json::Value>, // expanded when requested
    pub customer_details: Option<CustomerDetails>,
    pub customer_email: Option<String>,
}
