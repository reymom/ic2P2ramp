use std::collections::HashSet;

use crate::errors::{OrderError, Result};
use crate::model::types::{PaymentProvider, PaymentProviderType};

pub fn validate_offramper_providers_for_order(providers: &[PaymentProvider]) -> Result<()> {
    if providers.is_empty() {
        return Err(OrderError::InvalidInput("Provider list is empty".into()).into());
    }

    // duplicates (exact) check
    {
        let mut set = HashSet::new();
        for p in providers {
            if !set.insert(p) {
                return Err(OrderError::InvalidInput("Duplicate provider entry".into()).into());
            }
        }
    }

    // one-of-each for these types
    let mut seen_paypal = false;
    let mut seen_revolut = false;
    let mut seen_stripe = false;

    for p in providers {
        match p.provider_type() {
            PaymentProviderType::PayPal => {
                if seen_paypal {
                    return Err(
                        OrderError::InvalidInput("Multiple PayPal not allowed".into()).into(),
                    );
                }
                seen_paypal = true;
            }
            PaymentProviderType::Revolut => {
                if seen_revolut {
                    return Err(
                        OrderError::InvalidInput("Multiple Revolut not allowed".into()).into(),
                    );
                }
                seen_revolut = true;
            }
            PaymentProviderType::Stripe => {
                if seen_stripe {
                    return Err(
                        OrderError::InvalidInput("Multiple Stripe not allowed".into()).into(),
                    );
                }
                seen_stripe = true;
            }
            PaymentProviderType::Email => {
                return Err(OrderError::InvalidInput(
                    "Offramper provider cannot be of type email".into(),
                )
                .into());
            }
            PaymentProviderType::Crypto => {}
        }
    }
    Ok(())
}
