use std::collections::HashSet;

use candid::CandidType;
use serde::Deserialize;

use crate::errors::{RampError, Result};

#[derive(CandidType, Deserialize, Clone, Debug, Eq)]
pub enum PaymentProvider {
    PayPal { id: String },
    Revolut { id: String },
}

impl PartialEq for PaymentProvider {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PaymentProvider::PayPal { .. }, PaymentProvider::PayPal { .. }) => true,
            (PaymentProvider::Revolut { .. }, PaymentProvider::Revolut { .. }) => true,
            _ => false,
        }
    }
}

impl std::hash::Hash for PaymentProvider {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            PaymentProvider::PayPal { .. } => state.write_u8(0),
            PaymentProvider::Revolut { .. } => state.write_u8(1),
        }
    }
}

impl PaymentProvider {
    pub fn get_id(&self) -> &str {
        match self {
            PaymentProvider::PayPal { id } => id,
            PaymentProvider::Revolut { id } => id,
        }
    }

    pub fn get_type(&self) -> u8 {
        match self {
            PaymentProvider::PayPal { .. } => 0,
            PaymentProvider::Revolut { .. } => 1,
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

pub fn contains_provider_type(
    provider: &PaymentProvider,
    providers: &HashSet<PaymentProvider>,
) -> bool {
    providers.contains(&provider)
}
