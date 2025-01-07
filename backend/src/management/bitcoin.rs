use std::str::FromStr;

use bitcoin::{
    address::Address,
    hashes::{sha256, Hash},
    secp256k1::{ecdsa::Signature, Message, Secp256k1, XOnlyPublicKey},
    PublicKey,
};

use crate::model::{
    errors::{BlockchainError, Result, SystemError, UserError},
    types::AddressType,
};

pub fn verify_signature(
    btc_address: &str,
    message: &str,
    signature: &str,
    pubkey: &str,
) -> Result<()> {
    // Check if the public key corresponds to the address
    let pubkey = PublicKey::from_str(pubkey).map_err(|_| BlockchainError::InvalidPubKey)?;
    let address = Address::from_str(btc_address)
        .map_err(|_| BlockchainError::InvalidAddress(AddressType::Bitcoin))?
        .assume_checked();

    let xonly_pubkey = XOnlyPublicKey::from(pubkey);
    if !address.is_related_to_pubkey(&pubkey) || address.is_related_to_xonly_pubkey(&xonly_pubkey) {
        return Err(UserError::InvalidSignature.into());
    }

    // Verify message signature
    let sig = Signature::from_str(signature).map_err(|_| UserError::InvalidSignature)?;
    let hashed_message = sha256::Hash::hash(message.as_bytes());
    let msg = Message::from_digest_slice(&hashed_message.as_ref()).map_err(|e| {
        return SystemError::InvalidInput(format!(
            "Failed to parse signature's message hash: {}",
            e.to_string()
        ));
    })?;

    let secp = Secp256k1::verification_only();
    secp.verify_ecdsa(&msg, &sig, &pubkey.inner)
        .map_err(|_| UserError::InvalidSignature)?;

    Ok(())
}
