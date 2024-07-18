use std::collections::HashMap;

use candid::CandidType;
use serde::Deserialize;

use crate::errors::{RampError, Result};

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PaymentProviderType {
    PayPal,
    Revolut,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, Hash)]
pub struct PaymentProvider {
    pub provider_type: PaymentProviderType,
    pub id: String,
}

impl PartialEq for PaymentProvider {
    fn eq(&self, other: &Self) -> bool {
        self.provider_type == other.provider_type
    }
}

impl PaymentProvider {
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(RampError::InvalidInput(
                "Payment Provider ID is empty".to_string(),
            ));
        }
        Ok(())
    }
}

pub fn contains_provider_type(
    provider: &PaymentProvider,
    providers: &HashMap<PaymentProviderType, String>,
) -> bool {
    providers.get(&provider.provider_type).is_some()
}

pub fn calculate_fees(fiat_amount: u64, crypto_amount: u64) -> (u64, u64) {
    // Static strategy: 2% fee for the offramper, 0.5% for the admin
    let offramper_fee = fiat_amount / 50; // 2%
    let admin_fee = crypto_amount / 200; // 0.5%

    (offramper_fee, admin_fee)
}
