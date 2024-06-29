use candid::CandidType;
use serde::Deserialize;

use crate::errors::{RampError, Result};

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
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

    pub fn get_type(&self) -> &str {
        match self {
            PaymentProvider::PayPal { .. } => "PayPal",
            PaymentProvider::Revolut { .. } => "Revolut",
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.get_id().is_empty() {
            return Err(RampError::InvalidInput(
                "Payment Provider ID is empty".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn contains_provider_type(provider: PaymentProvider, providers: &[PaymentProvider]) -> bool {
    providers
        .iter()
        .any(|p| p.get_type() == provider.get_type())
}
