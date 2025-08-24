use std::str::FromStr;

use ic_cdk::api::call::call;
use icramp_types::bitcoin::errors::Result as BitcoinResult;

use crate::{
    Result,
    model::{errors::SystemError, memory::heap::read_state},
};
use bitcoin_backend::types::{RuneID, RuneMetadata, RuneUTXOEntry, TransactionType};

pub async fn bitcoin_backend_transfer(
    dst_address: String,
    amount: u64,
    tx_type: TransactionType,
    rune_utxos: Option<Vec<RuneUTXOEntry>>,
) -> Result<String> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    let (tx_id,): (BitcoinResult<String>,) = (call::<
        (String, u64, TransactionType, Option<Vec<RuneUTXOEntry>>),
        (BitcoinResult<String>,),
    >(
        bitcoin_backend_canister_id,
        "transfer",
        (dst_address, amount, tx_type, rune_utxos),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0,);

    Ok(tx_id?)
}

pub async fn bitcoin_backend_deposit_funds(
    offramper: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    call::<(String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "deposit_to_address_vault",
        (offramper, amount, rune),
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
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

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
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

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
) -> Result<()> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    call::<(String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "cancel_deposit",
        (offramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `complete_order` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_complete_order(
    onramper_address: String,
    amount: u64,
    rune: Option<RuneID>,
) -> Result<()> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    call::<(String, u64, Option<RuneID>), ()>(
        bitcoin_backend_canister_id,
        "complete_order",
        (onramper_address, amount, rune),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

/// Calls the `validate_rune_metadata` function on the `bitcoin_backend` canister.
pub async fn bitcoin_backend_validate_rune(rune_id: RuneID) -> Result<()> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    call::<(RuneID,), ()>(bitcoin_backend_canister_id, "validate_rune", (rune_id,))
        .await
        .map_err(|(code, err)| SystemError::ICRejectionError(code, err).into())
}

pub async fn bitcoin_backend_estimate_fee() -> Result<u64> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    let (fee,): (BitcoinResult<u64>,) = (call::<(), (BitcoinResult<u64>,)>(
        bitcoin_backend_canister_id,
        "estimate_bitcoin_transaction_fee",
        (),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0,);

    Ok(fee?)
}

pub async fn bitcoin_backend_get_rune_metadata(rune_id: String) -> Result<RuneMetadata> {
    let bitcoin_backend_canister_id = read_state(|s| s.canister_ids.bitcoin_backend_id);

    Ok(call::<(RuneID,), (BitcoinResult<(RuneMetadata, String)>,)>(
        bitcoin_backend_canister_id,
        "get_serialized_rune_metadata",
        (RuneID::from_str(&rune_id)?,),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0?
    .0)
}
