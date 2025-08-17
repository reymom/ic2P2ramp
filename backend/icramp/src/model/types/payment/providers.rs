use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use candid::{CandidType, Deserialize};

use crate::errors::{Result, SystemError};

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PaymentProviderType {
    PayPal,
    Revolut,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq)]
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

impl Hash for PaymentProvider {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.provider_type().hash(state);
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
                    return Err(SystemError::InvalidInput("Paypal ID is empty".to_string()).into());
                }
            }
            PaymentProvider::Revolut { scheme, id, .. } => {
                if scheme.is_empty() || id.is_empty() {
                    return Err(
                        SystemError::InvalidInput("Revolut details are empty".to_string()).into(),
                    );
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
