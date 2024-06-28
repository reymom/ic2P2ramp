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

    let user = User {
        evm_address: evm_address.clone(),
        payment_providers,
        offramped_amount: 0,
        score: 0,
    };

    storage::USERS.with(|p| p.borrow_mut().insert(evm_address.clone(), user));
    Ok(evm_address)
}

pub fn get_user(evm_address: String) -> Result<User> {
    storage::USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&evm_address) {
            Ok(user.clone())
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn add_payment_provider(evm_address: String, payment_provider: PaymentProvider) -> Result<()> {
    storage::USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(mut user) = users.remove(&evm_address) {
            let provider_type = match &payment_provider {
                PaymentProvider::PayPal { .. } => "PayPal",
                PaymentProvider::Revolut { .. } => "Revolut",
            };
            let mut replaced = false;
            for provider in &mut user.payment_providers {
                match provider {
                    PaymentProvider::PayPal { .. } if provider_type == "PayPal" => {
                        *provider = payment_provider.clone();
                        replaced = true;
                        break;
                    }
                    PaymentProvider::Revolut { .. } if provider_type == "Revolut" => {
                        *provider = payment_provider.clone();
                        replaced = true;
                        break;
                    }
                    _ => {}
                }
            }

            if !replaced {
                user.payment_providers.push(payment_provider);
            }

            users.insert(evm_address.clone(), user);
            Ok(())
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn can_commit_order(onramper_address: &str) -> Result<bool> {
    storage::USERS.with(|users| {
        let users = users.borrow();
        if let Some(user) = users.get(&onramper_address.to_string()) {
            Ok(user.can_commit_order())
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn increase_user_score(onramper_address: &str, fiat_amount: u64) -> Result<i32> {
    storage::USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(mut user) = users.remove(&onramper_address.to_string()) {
            user.increase_score(fiat_amount);
            let score = user.score;
            users.insert(onramper_address.to_string(), user);
            Ok(score)
        } else {
            Err(RampError::UserNotFound)
        }
    })
}

pub fn decrease_user_score(onramper_address: &str) -> Result<i32> {
    storage::USERS.with(|users| {
        let mut users = users.borrow_mut();
        if let Some(mut user) = users.remove(&onramper_address.to_string()) {
            user.decrease_score();
            let score = user.score;
            users.insert(onramper_address.to_string(), user);
            Ok(score)
        } else {
            Err(RampError::UserNotFound)
        }
    })
}
