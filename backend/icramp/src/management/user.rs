use std::collections::HashSet;

use super::random;
use crate::{
    model::{
        errors::{Result, SystemError, UserError},
        memory::stable::users,
    },
    types::{
        LoginAddress, PaymentProvider, TransactionAddress,
        session::Session,
        user::{User, UserType},
    },
};

pub async fn register_user(
    user_type: UserType,
    payment_providers: Vec<PaymentProvider>,
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

    assert_no_exact_duplicates(&payment_providers)?;
    assert_role_allows_list(&user_type, &payment_providers)?;

    for p in &payment_providers {
        p.validate().await?;
    }

    let mut user = User::new(user_type, login_address, hashed_password?)?;
    user.payment_providers = payment_providers.clone();
    assert_crypto_addresses_belong(&payment_providers, &user.addresses)?;

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

pub async fn add_payment_provider(
    user_id: u64,
    token: &str,
    payment_provider: PaymentProvider,
) -> Result<()> {
    payment_provider.validate().await?;

    users::mutate_user(user_id, |user| {
        user.validate_session(token)?;

        assert_can_add_provider(user, &payment_provider)?;

        user.payment_providers.push(payment_provider);
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

        user.payment_providers.pop_if(|p| p == payment_provider);
        Ok(())
    })?
}

pub fn update_user_auth_message(user_id: u64, auth_message: &str) -> Result<()> {
    users::mutate_user(user_id, |user| {
        user.auth_message = Some(auth_message.to_string());
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

// -------
// HELPERS
// -------

/// Purpose: returns Err if the vector contains exact duplicate providers.
fn assert_no_exact_duplicates(list: &[PaymentProvider]) -> Result<()> {
    use std::collections::HashSet;
    let mut set: HashSet<&PaymentProvider> = HashSet::new();
    for p in list {
        if !set.insert(p) {
            return Err(SystemError::InvalidInput("Duplicate payment provider".into()).into());
        }
    }
    Ok(())
}

/// Purpose: role constraints (Stripe only Offramper, Email only Onramper).
fn assert_role_allows_list(user_type: &UserType, list: &[PaymentProvider]) -> Result<()> {
    if matches!(user_type, UserType::Onramper)
        && list
            .iter()
            .any(|p| matches!(p, PaymentProvider::Stripe { .. }))
    {
        return Err(SystemError::InvalidInput(
            "Stripe is only allowed for Offramper users.".into(),
        )
        .into());
    }
    if matches!(user_type, UserType::Offramper)
        && list
            .iter()
            .any(|p| matches!(p, PaymentProvider::Email { .. }))
    {
        return Err(
            SystemError::InvalidInput("Email is only allowed for Onramper users.".into()).into(),
        );
    }
    Ok(())
}

/// Purpose: ensure all Crypto provider addresses belong to the given address book.
fn assert_crypto_addresses_belong(
    list: &[PaymentProvider],
    addresses: &HashSet<TransactionAddress>,
) -> Result<()> {
    for p in list {
        if let PaymentProvider::Crypto { address, .. } = p {
            if !addresses.contains(address) {
                return Err(SystemError::InvalidInput(
                    "Crypto provider address not present in user's addresses".into(),
                )
                .into());
            }
        }
    }
    Ok(())
}

/// Purpose: single-candidate version for add_payment_provider().
fn assert_can_add_provider(user: &User, candidate: &PaymentProvider) -> Result<()> {
    // role rules
    if matches!(user.user_type, UserType::Onramper)
        && matches!(candidate, PaymentProvider::Stripe { .. })
    {
        return Err(SystemError::InvalidInput(
            "Stripe is only allowed for Offramper users.".into(),
        )
        .into());
    }
    if matches!(user.user_type, UserType::Offramper)
        && matches!(candidate, PaymentProvider::Email { .. })
    {
        return Err(
            SystemError::InvalidInput("Email is only allowed for Onramper users.".into()).into(),
        );
    }
    // duplicate guard (exact)
    if user.payment_providers.iter().any(|p| p == candidate) {
        return Err(SystemError::InvalidInput("Duplicate payment provider".into()).into());
    }
    // crypto address ownership
    if let PaymentProvider::Crypto { address, .. } = candidate {
        if !user.addresses.contains(address) {
            return Err(SystemError::InvalidInput(
                "Crypto provider address not present in user's addresses".into(),
            )
            .into());
        }
    }
    Ok(())
}
