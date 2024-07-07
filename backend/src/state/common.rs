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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_contains_provider_type() {
//         let mut providers = HashSet::new();
//         providers.insert(PaymentProvider::PayPal {
//             id: "paypal_1".to_string(),
//         });
//         providers.insert(PaymentProvider::Revolut {
//             id: "revolut_1".to_string(),
//         });

//         let paypal_provider = PaymentProvider::PayPal {
//             id: "paypal_2".to_string(),
//         };
//         let revolut_provider = PaymentProvider::Revolut {
//             id: "revolut_2".to_string(),
//         };

//         assert!(contains_provider_type(&paypal_provider, &providers));
//         assert!(contains_provider_type(&revolut_provider, &providers));
//     }
// }
