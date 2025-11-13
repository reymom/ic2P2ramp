use std::collections::HashSet;

use crate::errors::{OrderError, Result};
use crate::model::types::{BlockchainAsset, PaymentProvider, PaymentProviderType};

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

pub fn has_same_chain_provider(
    offramper_providers: &[PaymentProvider],
    order_asset: &BlockchainAsset,
) -> bool {
    for provider in offramper_providers {
        if let PaymentProvider::Crypto {
            asset: provider_asset,
            ..
        } = provider
        {
            match (order_asset, provider_asset) {
                (
                    BlockchainAsset::EVM {
                        chain_id: order_chain,
                        ..
                    },
                    BlockchainAsset::EVM {
                        chain_id: provider_chain,
                        ..
                    },
                ) if order_chain == provider_chain => return true,
                (BlockchainAsset::ICP { .. }, BlockchainAsset::ICP { .. }) => return true,
                (BlockchainAsset::Bitcoin { .. }, BlockchainAsset::Bitcoin { .. }) => return true,
                (BlockchainAsset::Solana { .. }, BlockchainAsset::Solana { .. }) => return true,

                // Different chains - OK, continue checking
                _ => continue,
            }
        }
    }
    false
}
