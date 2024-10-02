use std::collections::HashSet;

use super::random;
use crate::{
    model::{
        errors::{Result, SystemError, UserError},
        memory::stable::users,
    },
    types::{
        session::Session,
        user::{User, UserType},
        LoginAddress, PaymentProvider, TransactionAddress,
    },
};

pub async fn register_user(
    user_type: UserType,
    payment_providers: HashSet<PaymentProvider>,
    login_address: LoginAddress,
    password: Option<String>,
) -> Result<User> {
    login_address.validate()?;

    let hashed_password: Result<Option<String>> = match login_address.clone() {
        LoginAddress::Email { .. } => {
            let password = password.ok_or(UserError::PasswordRequired)?;
            Ok(Some(random::hash_password(&password).await?))
        }
        LoginAddress::ICP { principal_id } => {
            ic_cdk::println!("[register] caller = {:?}", ic_cdk::caller().to_string());
            if ic_cdk::caller().to_string() != principal_id {
                return Err(UserError::UnauthorizedPrincipal)?;
            }
            Ok(None)
        }
        _ => Ok(None),
    };

    if payment_providers.is_empty() {
        return Err(SystemError::InvalidInput(
            "Provider list is empty.".to_string(),
        ))?;
    }
    payment_providers
        .clone()
        .into_iter()
        .try_for_each(|p| p.validate())?;

    let mut user = User::new(user_type, login_address, hashed_password?)?;
    user.payment_providers = payment_providers;

    users::insert_user(&user);
    Ok(user)
}

pub async fn reset_password_user(
    login_address: LoginAddress,
    new_password: Option<String>,
) -> Result<()> {
    login_address.validate()?;
    let hashed_password = if let LoginAddress::Email { .. } = login_address {
        let password = new_password.ok_or(UserError::PasswordRequired)?;
        random::hash_password(&password).await?
    } else {
        return Err(SystemError::InvalidInput(
            "Login Address must be of type Email".to_string(),
        ))?;
    };

    users::reset_password_user(&login_address, hashed_password)?;
    Ok(())
}

pub fn add_transaction_address(
    user_id: u64,
    token: &str,
    address: TransactionAddress,
) -> Result<()> {
    address.validate()?;

    users::mutate_user(user_id, |user| {
        user.validate_session(token)?;

        if let Some(existing_address) = user.addresses.take(&address) {
            ic_cdk::println!("updating address {:?} to {:?}", existing_address, address)
        }

        user.addresses.insert(address);
        Ok(())
    })?
}

pub fn add_payment_provider(
    user_id: u64,
    token: &str,
    payment_provider: PaymentProvider,
) -> Result<()> {
    payment_provider.validate()?;

    users::mutate_user(user_id, |user| {
        user.validate_session(token)?;

        user.payment_providers.insert(payment_provider);
        Ok(())
    })?
}

pub fn remove_payment_provider(
    user_id: u64,
    token: &str,
    payment_provider: &PaymentProvider,
) -> Result<()> {
    users::mutate_user(user_id, |user| {
        user.validate_session(token)?;

        user.payment_providers.remove(payment_provider);
        Ok(())
    })?
}

pub fn update_user_auth_message(user_id: u64, auth_message: &str) -> Result<()> {
    users::mutate_user(user_id, |user| {
        user.evm_auth_message = Some(auth_message.to_string());
    })
}

pub fn set_session(user_id: u64, session: &Session) -> Result<User> {
    users::mutate_user(user_id, |user| {
        user.session = Some(session.clone());
        Ok(user.to_owned())
    })?
}

pub fn update_onramper_payment(user_id: u64, fiat_amount: u64, currency: &str) -> Result<()> {
    users::mutate_user(user_id, |user| {
        user.update_fiat_amount(fiat_amount, currency);
        user.increase_score();
    })
}

pub fn update_offramper_payment(user_id: u64, fiat_amount: u64, currency: &str) -> Result<()> {
    users::mutate_user(user_id, |user| {
        user.update_fiat_amount(fiat_amount, currency)
    })
}
