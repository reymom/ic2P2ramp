use crate::model::errors::SystemError;
use crate::Result;
use candid::Principal;
use ic_cdk::api::call::call;

const BITCOIN_BACKEND_CANISTER_ID: &str = "zhuzm-wqaaa-aaaap-qpk2q-cai";

/// Calls the `lock_funds` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_lock_funds(
    offramper_address: String,
    onramper_address: String,
    amount: u64,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, String, u64), ()>(
        bitcoin_backend_canister_id,
        "lock_funds",
        (offramper_address, onramper_address, amount),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `unlock_funds` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_unlock_funds(
    offramper_address: String,
    onramper_address: String,
    amount: u64,
) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, String, u64), ()>(
        bitcoin_backend_canister_id,
        "unlock_funds",
        (offramper_address, onramper_address, amount),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `cancel_deposit` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_cancel_deposit(offramper_address: String, amount: u64) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, u64), ()>(
        bitcoin_backend_canister_id,
        "cancel_deposit",
        (offramper_address, amount),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `complete_order_and_send` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_send_funds(onramper_address: String, amount: u64) -> Result<()> {
    let bitcoin_backend_canister_id = Principal::from_text(BITCOIN_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    call::<(String, u64), ()>(
        bitcoin_backend_canister_id,
        "complete_order_and_send",
        (onramper_address, amount),
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
