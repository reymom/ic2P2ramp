use std::collections::HashSet;

use crate::{
    errors::{RampError, Result},
    state::storage::{self, PaymentProvider, User, UserType},
};

pub fn register_user(
    evm_address: String,
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
) -> Result<User> {
    if payment_providers.is_empty() {
        return Err(RampError::InvalidInput(
            "Provider list is empty.".to_string(),
        ));
    }
    payment_providers
        .clone()
        .into_iter()
        .try_for_each(|p| p.validate())?;

    if let Ok(_) = storage::get_user(&evm_address) {
        return Err(RampError::InvalidInput(
            "EVM address already registered.".to_string(),
        ));
    }

    let mut user = User::new(evm_address, user_type)?;
    user.payment_providers = payment_providers;

    storage::insert_user(&user);
    Ok(user)
}

pub fn add_payment_provider(evm_address: &str, payment_provider: PaymentProvider) -> Result<()> {
    payment_provider.validate()?;

    storage::mutate_user(evm_address, |user| {
        user.payment_providers.replace(payment_provider);
    })?;

    Ok(())
}

pub fn can_commit_orders(onramper_address: &str) -> Result<bool> {
    Ok(storage::get_user(&onramper_address)?.can_commit_orders())
}

pub fn update_onramper_payment(address: &str, fiat_amount: u64) -> Result<i32> {
    storage::mutate_user(&address, |user| {
        user.increase_score(fiat_amount);
        user.update_fiat_amount(fiat_amount);
        user.score
    })
}

pub fn update_offramper_payment(address: &str, fiat_amount: u64) -> Result<()> {
    storage::mutate_user(&address, |user| user.update_fiat_amount(fiat_amount))
}

pub fn decrease_user_score(onramper_address: &str) -> Result<i32> {
    storage::mutate_user(&onramper_address, |user| {
        user.decrease_score();
        user.score
    })
}
