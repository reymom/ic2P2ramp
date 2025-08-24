use crate::solana::errors::{Result, SolanaError, SystemError};
use ic_ed25519::{PublicKey, PublicKeyDecodingError, SignatureError};

pub fn verify_signature(
    address_b58: &str,
    message: &str,
    signature_b58: &str,
    pubkey_b58: &str,
) -> Result<()> {
    let sig_bytes = bs58::decode(signature_b58)
        .into_vec()
        .map_err(|e| SystemError::InvalidInput(format!("Invalid Solana signature b58: {e}")))?;
    let pk_bytes = bs58::decode(pubkey_b58)
        .into_vec()
        .map_err(|e| SystemError::InvalidInput(format!("Invalid Solana pubkey b58: {e}")))?;

    // Address must equal pubkey (base58)
    if address_b58 != pubkey_b58 {
        return Err(SystemError::InvalidInput("Address is not equal to pubkey".to_string()).into());
    }

    let pk = PublicKey::deserialize_raw(&pk_bytes).map_err(|e: PublicKeyDecodingError| {
        SystemError::InvalidInput(format!("Bad Solana pubkey: {e}"))
    })?;

    if !pk.is_canonical() {
        return Err(SystemError::InvalidInput("Non‑canonical Solana pubkey".to_string()).into());
    }
    if !pk.is_torsion_free() {
        return Err(SystemError::InvalidInput(
            "Solana pubkey not in prime order subgroup".to_string(),
        )
        .into());
    }

    // Verify (ZIP215 semantics inside ic-ed25519)
    pk.verify_signature(message.as_bytes(), &sig_bytes)
        .map_err(|_e: SignatureError| SolanaError::InvalidSignature)?;

    Ok(())
}
