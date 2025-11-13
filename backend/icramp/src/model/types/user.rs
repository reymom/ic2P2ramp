use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_stable_structures::{Storable, storable::Bound};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use super::{
    AuthenticationData, PaymentProvider,
    common::{LoginAddress, TransactionAddress},
    session::Session,
};
use crate::{
    errors::{BlockchainError, Result, SystemError, UserError},
    evm::signer,
    management::{bitcoin, random},
    model::{
        memory,
        types::{AddressType, PaymentProviderType},
    },
};

const MAX_USER_SIZE: u32 = 1500;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum UserType {
    Offramper,
    Onramper,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct User {
    pub id: u64,
    pub user_type: UserType,
    pub payment_providers: Vec<PaymentProvider>,
    pub addresses: HashSet<TransactionAddress>,
    pub fiat_amounts: HashMap<String, u64>, // offramped or onramped funds
    pub score: i32,
    pub login: LoginAddress,
    pub hashed_password: Option<String>, // for email login
    pub auth_message: Option<String>,    // for EVM & Bitcoin login, unique per session
    pub session: Option<Session>,
}

impl User {
    pub fn new(
        user_type: UserType,
        login_address: LoginAddress,
        hashed_password: Option<String>,
    ) -> Result<Self> {
        login_address.validate()?;

        let mut addresses = HashSet::new();
        if let LoginAddress::Email { .. } = login_address {
        } else {
            addresses.insert(login_address.to_transaction_address()?);
        };

        Ok(Self {
            id: memory::heap::generate_user_id(),
            user_type,
            payment_providers: Vec::new(),
            fiat_amounts: HashMap::new(),
            score: 1,
            login: login_address,
            hashed_password,
            auth_message: None,
            addresses,
            session: None,
        })
    }

    pub fn is_offramper(&self) -> Result<()> {
        match self.user_type {
            UserType::Offramper => Ok(()),
            UserType::Onramper => Err(UserError::UserNotOfframper.into()),
        }
    }

    pub fn validate_onramper(&self) -> Result<()> {
        match self.user_type {
            UserType::Onramper => Ok(()),
            UserType::Offramper => Err(UserError::UserNotOnramper.into()),
        }
    }

    pub fn verify_user_auth(&self, auth_data: Option<AuthenticationData>) -> Result<()> {
        match &self.login {
            LoginAddress::Email { .. } => {
                let password = auth_data
                    .ok_or(UserError::PasswordRequired)?
                    .password
                    .ok_or(UserError::PasswordRequired)?;
                let hashed_password =
                    self.hashed_password
                        .clone()
                        .ok_or(SystemError::InternalError(
                            "Password not in user".to_string(),
                        ))?;
                match random::verify_password(&password, &hashed_password) {
                    Ok(true) => {
                        return Ok(());
                    }
                    Ok(false) => {
                        return Err(UserError::InvalidPassword.into());
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            LoginAddress::ICP { principal_id } => {
                ic_cdk::println!(
                    "[verify_user_auth] caller = {:?}",
                    ic_cdk::caller().to_string()
                );
                ic_cdk::println!("[verify_user_auth] principal_id = {:?}", principal_id);
                if ic_cdk::caller()
                    != Principal::from_text(principal_id)
                        .map_err(|_| BlockchainError::InvalidAddress(AddressType::ICP))?
                {
                    return Err(UserError::UnauthorizedPrincipal.into());
                }
            }
            LoginAddress::EVM { address } => {
                let signature = auth_data
                    .clone()
                    .ok_or(UserError::SignatureRequired)?
                    .signature
                    .ok_or(UserError::SignatureRequired)?;
                let message = self.auth_message.as_ref().ok_or_else(|| {
                    SystemError::InternalError("evm auth message not in user".to_string())
                })?;

                signer::verify_signature(address, message, &signature)?
            }
            LoginAddress::Bitcoin { address } => {
                let signature = auth_data
                    .clone()
                    .ok_or(UserError::SignatureRequired)?
                    .signature
                    .ok_or(UserError::SignatureRequired)?;
                let pubkey = auth_data
                    .ok_or(UserError::PublicKeyRequired)?
                    .pubkey
                    .ok_or(UserError::PublicKeyRequired)?;
                let message = self.auth_message.as_ref().ok_or_else(|| {
                    SystemError::InternalError("bitcoin auth message not in user".to_string())
                })?;

                bitcoin::verify_signature(address, message, &signature, &pubkey)?
            }
            LoginAddress::Solana { address } => {
                let signature = auth_data
                    .clone()
                    .ok_or(UserError::SignatureRequired)?
                    .signature
                    .ok_or(UserError::SignatureRequired)?;
                let pubkey = auth_data
                    .ok_or(UserError::PublicKeyRequired)?
                    .pubkey
                    .ok_or(UserError::PublicKeyRequired)?;
                let message = self.auth_message.as_ref().ok_or_else(|| {
                    SystemError::InternalError("solana auth message not in user".to_string())
                })?;

                icramp_types::solana::verify_signature(address, message, &signature, &pubkey)?
            }
        }

        Ok(())
    }

    pub fn validate_session(&self, token: &str) -> Result<()> {
        self.session
            .as_ref()
            .ok_or(UserError::SessionNotFound)?
            .validate(token)
    }

    pub fn update_fiat_amount(&mut self, amount: u64, currency: &str) {
        *self.fiat_amounts.entry(currency.to_string()).or_insert(0) += amount;
    }

    pub fn decrease_score(&mut self) {
        self.score -= 1;
    }

    pub fn increase_score(&mut self) {
        self.score += 1;
    }

    pub fn is_banned(&self) -> Result<()> {
        if self.score < 0 {
            return Err(UserError::UserBanned.into());
        }
        Ok(())
    }
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_USER_SIZE,
        is_fixed_size: false,
    };
}

fn contains_provider_exact(needle: &PaymentProvider, haystack: &[PaymentProvider]) -> bool {
    haystack.iter().any(|p| p == needle)
}

/// Purpose: ensure `subset` ⊆ `superset` (exact entries); returns first missing provider type if any.
pub fn first_missing_provider<'a>(
    subset: &'a [PaymentProvider],
    superset: &'a [PaymentProvider],
) -> Option<PaymentProviderType> {
    subset
        .iter()
        .find(|p| !contains_provider_exact(p, superset))
        .map(|p| p.provider_type())
}
