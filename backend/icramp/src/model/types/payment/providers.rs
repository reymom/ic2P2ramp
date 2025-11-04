use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use candid::{CandidType, Deserialize};

use crate::{
    errors::{Result, SystemError},
    model::{helpers::validate_email, types::payment::stripe::pick_platform},
    outcalls::stripe::account::get_account_info,
};

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PaymentProviderType {
    PayPal,
    Revolut,
    Stripe,
    Email,
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
    Stripe {
        account_id: String,
        platform: String,
    },
    Email {
        email: String,
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
            PaymentProvider::Stripe { .. } => PaymentProviderType::Stripe,
            PaymentProvider::Email { .. } => PaymentProviderType::Email,
        }
    }

    pub async fn validate(&self) -> Result<()> {
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
            PaymentProvider::Stripe {
                account_id,
                platform,
            } => {
                if !account_id.starts_with("acct_") {
                    return Err(
                        SystemError::InvalidInput("Invalid Stripe account_id".into()).into(),
                    );
                }
                let _ = pick_platform(Some(platform.clone()))?;
                verify_stripe_acct(account_id, platform).await?;
            }
            PaymentProvider::Email { email } => validate_email(email)?,
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

async fn verify_stripe_acct(account_id: &str, platform: &str) -> Result<()> {
    let info = get_account_info(account_id, Some(platform.to_string()))
        .await
        .map_err(|e| SystemError::InternalError(format!("Stripe info fetch failed: {e}")))?;

    let payouts_ok = info.payouts_enabled == true;
    let transfers_ok = matches!(info.capabilities.transfers.as_str(), "active");

    if !(payouts_ok && transfers_ok) {
        return Err(SystemError::InvalidInput(
            "Stripe account not ready (require payouts_enabled && transfers=active)".into(),
        ))?;
    }
    Ok(())
}
