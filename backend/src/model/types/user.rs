use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashSet};

use crate::{
    errors::{RampError, Result},
    management::random,
};

use super::{
    common::{Address, PaymentProvider},
    AddressType,
};

const MAX_USER_SIZE: u32 = 1000;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum UserType {
    Offramper,
    Onramper,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct User {
    pub user_type: UserType,
    pub payment_providers: HashSet<PaymentProvider>,
    pub fiat_amount: u64, // received for offramper or payed by onramper
    pub score: i32,
    pub login_method: Address,
    pub hashed_password: Option<String>,
    pub addresses: HashSet<Address>,
}

impl User {
    pub fn new(
        user_type: UserType,
        initial_address: Address,
        hashed_password: Option<String>,
    ) -> Result<Self> {
        let mut addresses = HashSet::new();
        initial_address.validate()?;
        if initial_address.address_type != AddressType::Email {
            addresses.insert(initial_address.clone());
        }

        Ok(Self {
            user_type,
            payment_providers: HashSet::new(),
            fiat_amount: 0,
            score: 1,
            login_method: initial_address,
            hashed_password,
            addresses,
        })
    }

    pub fn validate_offramper(&self) -> Result<()> {
        match self.user_type {
            UserType::Offramper => Ok(()),
            UserType::Onramper => Err(RampError::UserNotOfframper),
        }
    }

    pub fn validate_onramper(&self) -> Result<()> {
        match self.user_type {
            UserType::Onramper => Ok(()),
            UserType::Offramper => Err(RampError::UserNotOnramper),
        }
    }

    pub fn verify_user_password(&self, password: Option<String>) -> Result<()> {
        if self.login_method.address_type == AddressType::Email {
            let password = password.ok_or(RampError::PasswordRequired)?;

            if let Some(ref stored_password) = &self.hashed_password {
                match random::verify_password(&password, &stored_password) {
                    Ok(true) => {
                        return Ok(());
                    }
                    Ok(false) => {
                        return Err(RampError::InvalidPassword);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            } else {
                return Err(RampError::InternalError(
                    "No stored password found for verification.".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub fn update_fiat_amount(&mut self, amount: u64) {
        self.fiat_amount += amount;
    }

    pub fn decrease_score(&mut self) {
        self.score -= 1;
    }

    pub fn increase_score(&mut self, amount: u64) {
        self.score += (amount / 1000) as i32; // Assuming amount is in cents
    }

    pub fn is_banned(&self) -> Result<()> {
        if self.score < 0 {
            return Err(RampError::UserBanned);
        }
        Ok(())
    }
}

impl Storable for User {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_USER_SIZE,
        is_fixed_size: false,
    };
}
