use std::collections::HashSet;

use super::random;
use crate::{
    errors::{RampError, Result},
    state::storage,
    types::{
        user::{User, UserType},
        LoginAddress, PaymentProvider, TransactionAddress,
    },
};

pub async fn register_user(
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
    mut login_address: LoginAddress,
) -> Result<User> {
    login_address.validate()?;

    if let LoginAddress::Email {
        ref mut password, ..
    } = login_address
    {
        let hashed_password = random::hash_password(password).await?;
        *password = hashed_password; // Replace the plaintext password with the hashed one
    }

    if payment_providers.is_empty() {
        return Err(RampError::InvalidInput(
            "Provider list is empty.".to_string(),
        ));
    }
    payment_providers
        .clone()
        .into_iter()
        .try_for_each(|p| p.validate())?;

    let mut user = User::new(user_type, login_address)?;
    user.payment_providers = payment_providers;

    storage::insert_user(&user);
    Ok(user)
}

pub fn add_transaction_address(user_id: u64, address: TransactionAddress) -> Result<()> {
    address.validate()?;

    storage::mutate_user(user_id, |user| {
        if let Some(existing_address) = user.addresses.take(&address) {
            ic_cdk::println!("updating address {:?} to {:?}", existing_address, address)
        }

        user.addresses.insert(address);
        Ok(())
    })?
}

pub fn add_payment_provider(user_id: u64, payment_provider: PaymentProvider) -> Result<()> {
    payment_provider.validate()?;

    storage::mutate_user(user_id, |user| {
        user.payment_providers.insert(payment_provider);
    })?;

    Ok(())
}

pub fn can_commit_orders(user_id: &u64) -> Result<()> {
    let user = storage::get_user(user_id)?;
    user.is_banned()?;
    user.validate_onramper()?;
    Ok(())
}

pub fn update_onramper_payment(user_id: u64, fiat_amount: u64) -> Result<i32> {
    storage::mutate_user(user_id, |user| {
        user.increase_score(fiat_amount);
        user.update_fiat_amount(fiat_amount);
        user.score
    })
}

pub fn update_offramper_payment(user_id: u64, fiat_amount: u64) -> Result<()> {
    storage::mutate_user(user_id, |user| user.update_fiat_amount(fiat_amount))
}

pub fn decrease_user_score(user_id: u64) -> Result<i32> {
    storage::mutate_user(user_id, |user| {
        user.decrease_score();
        user.score
    })
}
