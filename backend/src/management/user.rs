use std::collections::HashSet;

use super::random;
use crate::{
    errors::{RampError, Result},
    state::storage,
};
use crate::{
    model::types::AddressType,
    types::{
        user::{User, UserType},
        Address, PaymentProvider,
    },
};

pub async fn register_user(
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
    login_address: Address,
    password: Option<String>,
) -> Result<User> {
    login_address.validate()?;

    if login_address.address_type == AddressType::Email && password == None {
        return Err(RampError::PasswordRequired);
    }

    let hashed_password = if let Some(plaintext_password) = password {
        Some(random::hash_password(&plaintext_password).await?)
    } else {
        None
    };

    if payment_providers.is_empty() {
        return Err(RampError::InvalidInput(
            "Provider list is empty.".to_string(),
        ));
    }
    payment_providers
        .clone()
        .into_iter()
        .try_for_each(|p| p.validate())?;

    let mut user = User::new(user_type, login_address, hashed_password)?;
    user.payment_providers = payment_providers;

    storage::insert_user(&user);
    Ok(user)
}

pub fn add_address(login_address: &Address, address: Address) -> Result<()> {
    if *login_address == address {
        return Err(RampError::InvalidInput(
            "Login Address cannot be modified".to_string(),
        ));
    }
    address.validate()?;

    storage::mutate_user(login_address, |user| {
        if let Some(existing_address) = user.addresses.take(&address) {
            ic_cdk::println!("updating address {:?} to {:?}", existing_address, address)
        }

        user.addresses.insert(address);
        Ok(())
    })?
}

pub fn add_payment_provider(address: &Address, payment_provider: PaymentProvider) -> Result<()> {
    payment_provider.validate()?;

    storage::mutate_user(address, |user| {
        user.payment_providers.insert(payment_provider);
    })?;

    Ok(())
}

pub fn can_commit_orders(address: &Address) -> Result<()> {
    let user = storage::get_user(address)?;
    user.is_banned()?;
    user.validate_onramper()?;
    Ok(())
}

pub fn update_onramper_payment(address: &Address, fiat_amount: u64) -> Result<i32> {
    storage::mutate_user(address, |user| {
        user.increase_score(fiat_amount);
        user.update_fiat_amount(fiat_amount);
        user.score
    })
}

pub fn update_offramper_payment(address: &Address, fiat_amount: u64) -> Result<()> {
    storage::mutate_user(address, |user| user.update_fiat_amount(fiat_amount))
}

pub fn decrease_user_score(address: &Address) -> Result<i32> {
    storage::mutate_user(address, |user| {
        user.decrease_score();
        user.score
    })
}
