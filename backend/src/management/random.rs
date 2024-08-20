use pbkdf2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Pbkdf2,
};

use crate::model::errors::{RampError, Result};

pub async fn get_random_bytes() -> Result<[u8; 32]> {
    let management_canister = candid::Principal::management_canister();
    let rnd_buffer: (Vec<u8>,) = match ic_cdk::call(management_canister, "raw_rand", ()).await {
        Ok(result) => result,
        Err((code, msg)) => {
            ic_cdk::println!("Error invoking raw_rand: {:?} {}", code, msg);
            return Err(RampError::ICRejectionError(code, msg));
        }
    };

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&rnd_buffer.0[..32]);
    Ok(seed)
}

pub async fn hash_password(password: &str) -> Result<String> {
    // let salt = get_random_bytes().await?;
    // let mut hash = [0u8; 32];

    let random_bytes = get_random_bytes().await?;
    // Convert the random bytes to a base64 encoded string to create a SaltString
    // let salt_base64 = hex::encode(random_bytes);
    let salt = SaltString::encode_b64(&random_bytes)
        .map_err(|e| RampError::InternalError(e.to_string()))?;

    let password_hash = Pbkdf2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| RampError::InternalError(e.to_string()))?;

    // pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, 10_000, &mut hash);

    // ic_cdk::println!("Generated salt (hex): {}", hex::encode(&salt));
    // ic_cdk::println!("Generated hash (hex): {}", hex::encode(&hash));

    // Ok(format!("{}:{}", hex::encode(salt), hex::encode(hash)))

    Ok(password_hash.to_string())
}

// pub fn verify_password(password: &str, salt: &str, hash: &str) -> Result<bool> {
//     let salt_bytes =
//         hex::decode(salt).map_err(|_| RampError::InvalidInput("Invalid salt format".into()))?;
//     let stored_hash_bytes =
//         hex::decode(hash).map_err(|_| RampError::InvalidInput("Invalid hash format".into()))?;
//     let mut hash_bytes = [0u8; 32];

//     pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt_bytes, 10_000, &mut hash_bytes);

//     ic_cdk::println!("Input password: {}", password);
//     ic_cdk::println!("Salt (hex): {}", salt);
//     ic_cdk::println!("Computed hash (hex): {}", hex::encode(hash_bytes));
//     ic_cdk::println!("Stored hash (hex): {}", hash);
//     Ok(stored_hash_bytes == hash_bytes)
// }

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| RampError::InvalidInput(e.to_string()))?;
    Pbkdf2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .map_err(|_| RampError::InvalidPassword)
}
