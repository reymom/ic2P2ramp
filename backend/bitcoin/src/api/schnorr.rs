use candid::Principal;

use crate::model::{
    types::{
        errors::Result,
        schnorr::{
            SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKey, SchnorrPublicKeyReply,
            SignWithSchnorr, SignWithSchnorrReply,
        },
    },
    utils,
};

const SIGN_WITH_SCHNORR_FEE: u128 = 25_000_000_000;

/// Returns the Schnorr public key of this canister at the given derivation path.
pub async fn schnorr_public_key(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
) -> Result<Vec<u8>> {
    let payload = SchnorrPublicKey {
        canister_id: None,
        derivation_path,
        key_id: SchnorrKeyId {
            name: key_name,
            algorithm: SchnorrAlgorithm::Bip340Secp256k1,
        },
    };

    let res: std::result::Result<(SchnorrPublicKeyReply,), _> = ic_cdk::call(
        Principal::management_canister(),
        "schnorr_public_key",
        (payload,),
    )
    .await;

    let reply = utils::handle_ic_call(res).await?;
    Ok(reply.public_key)
}

pub async fn sign_with_schnorr(
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    message: Vec<u8>,
) -> Result<Vec<u8>> {
    let payload = SignWithSchnorr {
        message,
        derivation_path,
        key_id: SchnorrKeyId {
            name: key_name,
            algorithm: SchnorrAlgorithm::Bip340Secp256k1,
        },
    };

    let res: std::result::Result<(SignWithSchnorrReply,), _> =
        ic_cdk::api::call::call_with_payment128(
            Principal::management_canister(),
            "sign_with_schnorr",
            (payload,),
            SIGN_WITH_SCHNORR_FEE,
        )
        .await;

    let reply = utils::handle_ic_call(res).await?;
    Ok(reply.signature)
}
