use candid::CandidType;
use serde::Deserialize;

#[derive(CandidType, Deserialize, Clone)]
pub enum PaymentProvider {
    PayPal { id: String },
    Revolut { id: String },
}

impl PaymentProvider {
    pub fn get_id(&self) -> &str {
        match self {
            PaymentProvider::PayPal { id } => id,
            PaymentProvider::Revolut { id } => id,
        }
    }
}
