use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct StripeCapabilities {
    pub card_payments: String, // "active" | "inactive" | "pending"
    pub transfers: String,     // "active" | "inactive" | "pending"
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct StripeAccountInfo {
    pub id: String,
    pub country: String,
    pub default_currency: Option<String>,
    pub payouts_enabled: bool,
    pub charges_enabled: bool,
    pub details_submitted: bool,
    pub capabilities: StripeCapabilities,
}
