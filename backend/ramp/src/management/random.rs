use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use crate::model::errors::{Result, SystemError, UserError};

pub async fn get_random_bytes() -> Result<[u8; 32]> {
    let management_canister = candid::Principal::management_canister();
    let rnd_buffer: (Vec<u8>,) = match ic_cdk::call(management_canister, "raw_rand", ()).await {
        Ok(result) => result,
        Err((code, msg)) => {
            ic_cdk::println!("Error invoking raw_rand: {:?} {}", code, msg);
            return Err(SystemError::ICRejectionError(code, msg))?;
        }
    };

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&rnd_buffer.0[..32]);
    Ok(seed)
}

pub async fn hash_password(password: &str) -> Result<String> {
    let random_bytes = get_random_bytes().await?;
    // Convert the random bytes to a base64 encoded string to create a SaltString
    let salt = SaltString::encode_b64(&random_bytes)
        .map_err(|e| SystemError::InternalError(e.to_string()))?;

    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| SystemError::InternalError(e.to_string()))?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| SystemError::InvalidInput(e.to_string()))?;
    Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .map_err(|_| UserError::InvalidPassword.into())
}

pub async fn generate_token() -> Result<String> {
    let random_bytes = get_random_bytes().await?;
    Ok(hex::encode(random_bytes))
}
