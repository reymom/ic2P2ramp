use candid::Principal;
use ic_cdk::api::call::call;

use crate::model::errors::SystemError;
use crate::Result;
use bitcoin_backend::types::{RuneID, RuneUTXOEntry};

const BITCOIN_BACKEND_CANISTER_ID: &str = "zhuzm-wqaaa-aaaap-qpk2q-cai";

// todo: we need to execute this in backend/src/lib.rs:create_order
pub async fn bitcoin_backend_deposit_funds(
    offramper: String,
    amount: u64,
    rune: Option<RuneID>,
    utxos: Vec<RuneUTXOEntry>,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, u64, Option<RuneID>, Vec<RuneUTXOEntry>), ()>(
        bitcoin_backend_canister_id,
        "deposit_to_address_vault",
        (offramper, amount, rune, utxos),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `lock_funds` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_lock_funds(
    offramper_address: String,
    onramper_address: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "lock_funds",
        (offramper_address, onramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `unlock_funds` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_unlock_funds(
    offramper_address: String,
    onramper_address: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "unlock_funds",
        (offramper_address, onramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `cancel_deposit` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_cancel_deposit(
    offramper_address: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<String> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    let result: (String,) = call::<(String, u64, Option<RuneID>), (String,)>(
        bitcoin_backend_canister_id,
        "cancel_deposit",
        (offramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(result.0)
}

/// Calls the `complete_order_and_send` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_send_funds(
    onramper_address: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "complete_order_and_send",
        (onramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `validate_rune_metadata` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_validate_rune(rune_data: String) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String,), ()>(
        bitcoin_backend_canister_id,
        "validate_rune_metadata",
        (rune_data,),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

pub async fn bitcoin_backend_estimate_fee() -> Result<u64> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid Bitcoin backend principal".to_string()))?;

    call::<(), (u64,)>(
        bitcoin_backend_canister_id,
        "estimate_bitcoin_transaction_fee",
        (),
    )
    .await
    .map(|(fee,)| fee)
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}
