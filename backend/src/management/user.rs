use crate::{
    errors::{RampError, Result},
    state::storage::{self, PaymentProvider, User},
};

pub fn register_user(
    evm_address: String,
    payment_providers: Vec<PaymentProvider>,
) -> Result<String> {
    if payment_providers.is_empty() {
        return Err(RampError::InvalidInput(
            "Provider list is empty.".to_string(),
        ));
    }
    payment_providers.iter().try_for_each(|p| p.validate())?;

    if let Ok(_) = storage::get_user(&evm_address) {
        return Err(RampError::InvalidInput(
            "EVM address already registered.".to_string(),
        ));
    }

    let mut user = User::new(evm_address.clone())?;
    user.payment_providers = payment_providers;

    Ok(storage::insert_user(&user))
}

pub fn add_payment_provider(evm_address: String, payment_provider: PaymentProvider) -> Result<()> {
    payment_provider.validate()?;

    let mut user = storage::get_user(&evm_address)?;

    let provider_type = payment_provider.get_type();
    let mut replaced = false;
    for provider in &mut user.payment_providers {
        if provider.get_type() == provider_type {
            *provider = payment_provider.clone();
            replaced = true;
            break;
        }
    }
    if !replaced {
        user.payment_providers.push(payment_provider);
    }

    storage::insert_user(&user);
    Ok(())
}

pub fn can_commit_orders(onramper_address: &str) -> Result<bool> {
    Ok(storage::get_user(&onramper_address)?.can_commit_orders())
}

pub fn increase_user_score(onramper_address: &str, fiat_amount: u64) -> Result<i32> {
    let mut user = storage::get_user(&onramper_address)?;
    user.increase_score(fiat_amount);
    let score = user.score.clone();
    storage::insert_user(&user);
    Ok(score)
}

pub fn decrease_user_score(onramper_address: &str) -> Result<i32> {
    let mut user = storage::get_user(&onramper_address)?;
    user.decrease_score();
    let score = user.score.clone();
    storage::insert_user(&user);
    Ok(score)
}
