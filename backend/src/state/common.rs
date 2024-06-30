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
            (PaymentProvider::PayPal { id: id1 }, PaymentProvider::PayPal { id: id2 }) => {
                id1 == id2
            }
            (PaymentProvider::Revolut { id: id1 }, PaymentProvider::Revolut { id: id2 }) => {
                id1 == id2
            }
            _ => false,
        }
    }
}

impl std::hash::Hash for PaymentProvider {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        self.get_id().hash(state);
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
    providers
        .iter()
        .any(|p| p.get_type() == provider.get_type())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_provider_type() {
        let mut providers = HashSet::new();
        providers.insert(PaymentProvider::PayPal {
            id: "paypal_1".to_string(),
        });
        providers.insert(PaymentProvider::Revolut {
            id: "revolut_1".to_string(),
        });

        let paypal_provider = PaymentProvider::PayPal {
            id: "paypal_2".to_string(),
        };
        let revolut_provider = PaymentProvider::Revolut {
            id: "revolut_2".to_string(),
        };

        assert!(contains_provider_type(&paypal_provider, &providers));
        assert!(contains_provider_type(&revolut_provider, &providers));
    }
}
