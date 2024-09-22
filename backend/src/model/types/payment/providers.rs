use std::collections::HashMap;

use candid::{CandidType, Deserialize};

use crate::model::errors::{RampError, Result};

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PaymentProviderType {
    PayPal,
    Revolut,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, Hash)]
pub enum PaymentProvider {
    PayPal {
        id: String,
    },
    Revolut {
        scheme: String,
        id: String,
        name: Option<String>,
    },
}

impl PartialEq for PaymentProvider {
    fn eq(&self, other: &Self) -> bool {
        self.provider_type() == other.provider_type()
    }
}

impl PaymentProvider {
    pub fn provider_type(&self) -> PaymentProviderType {
        match self {
            PaymentProvider::PayPal { .. } => PaymentProviderType::PayPal,
            PaymentProvider::Revolut { .. } => PaymentProviderType::Revolut,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            PaymentProvider::PayPal { id } => {
                if id.is_empty() {
                    return Err(RampError::InvalidInput("Paypal ID is empty".to_string()));
                }
            }
            PaymentProvider::Revolut { scheme, id, .. } => {
                if scheme.is_empty() || id.is_empty() {
                    return Err(RampError::InvalidInput(
                        "Revolut details are empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

pub fn contains_provider_type(
    provider: &PaymentProvider,
    providers: &HashMap<PaymentProviderType, PaymentProvider>,
) -> bool {
    providers.get(&provider.provider_type()).is_some()
}
