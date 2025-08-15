use crate::errors::{Result, UserError};
use crate::types::{user::User, LoginAddress};

use super::storage::USERS;

pub fn mutate_user<F, R>(user_id: u64, f: F) -> Result<R>
where
    F: FnOnce(&mut User) -> R,
{
    USERS.with_borrow_mut(|users| {
        if let Some(mut user) = users.get(&user_id) {
            let result = f(&mut user);
            users.insert(user_id, user);
            Ok(result)
        } else {
            Err(UserError::UserNotFound.into())
        }
    })
}

pub fn insert_user(user: &User) -> Option<User> {
    USERS.with_borrow_mut(|p| p.insert(user.id, user.clone()))
}

pub fn remove_user(user_id: &u64) -> Result<User> {
    USERS
        .with_borrow_mut(|p| p.remove(user_id))
        .ok_or_else(|| UserError::UserNotFound.into())
}

pub fn get_user(user_id: &u64) -> Result<User> {
    USERS
        .with_borrow(|users| users.get(user_id))
        .ok_or_else(|| UserError::UserNotFound.into())
}

pub fn find_user_by_login_address(login_address: &LoginAddress) -> Result<u64> {
    USERS.with(|users| {
        for (id, user) in users.borrow().iter() {
            if user.login == *login_address {
                return Ok(id);
            }
        }
        Err(UserError::UserNotFound.into())
    })
}

pub fn reset_password_user(login_address: &LoginAddress, password: String) -> Result<u64> {
    USERS.with_borrow_mut(|users| {
        let mut user_to_update = None;

        for (_, user) in users.iter() {
            if user.login == *login_address {
                user_to_update = Some(User {
                    hashed_password: Some(password.clone()),
                    ..user
                })
            }
        }

        if let Some(user) = user_to_update {
            let id = user.id;
            users.insert(id, user);
            Ok(id)
        } else {
            Err(UserError::UserNotFound.into())
        }
    })
}
