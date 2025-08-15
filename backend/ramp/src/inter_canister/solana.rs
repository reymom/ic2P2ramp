use candid::{Nat, Principal};
use ic_cdk::call;

use crate::model::errors::{Result, SystemError};
use ramp_types::solana::errors::Result as SolanaResult;

const SOLANA_BACKEND_CANISTER_ID: &str = "u6s2n-gx777-77774-qaaba-cai";

pub async fn solana_backend_send_sol(dst: String, lamports: Nat) -> Result<String> {
    let solana_backend_canister_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    let (tx_id,): (SolanaResult<String>,) = (call::<
        (Option<Principal>, String, Nat),
        (SolanaResult<String>,),
    >(
        solana_backend_canister_id,
        "send_sol",
        (Some(ic_cdk::id()), dst, lamports),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0,);

    Ok(tx_id?)
}

pub async fn solana_backend_send_spl_token(
    mint_account: String,
    to: String,
    amount: Nat,
) -> Result<String> {
    let solana_backend_canister_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid ledger principal".to_string()))?;

    let (tx_id,): (SolanaResult<String>,) = (call::<
        (Option<Principal>, String, String, Nat),
        (SolanaResult<String>,),
    >(
        solana_backend_canister_id,
        "send_spl_token",
        (Some(ic_cdk::id()), mint_account, to, amount),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0,);

    Ok(tx_id?)
}

pub async fn solana_backend_deposit_funds(
    offramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid solana_backend principal".to_string()))?;

    let (res,): (SolanaResult<()>,) = call::<(String, u64, Option<String>), _>(
        can_id,
        "deposit_to_vault_canister",
        (offramper, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}

pub async fn solana_backend_cancel_deposit(
    offramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid solana_backend principal".to_string()))?;

    let (res,): (SolanaResult<()>,) = call::<(String, u64, Option<String>), _>(
        can_id,
        "cancel_deposit",
        (offramper, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}

pub async fn solana_backend_lock_funds(
    offramper: String,
    onramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid solana_backend principal".to_string()))?;

    let (res,): (SolanaResult<()>,) = call::<(String, String, u64, Option<String>), _>(
        can_id,
        "lock_funds",
        (offramper, onramper, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}

pub async fn solana_backend_unlock_funds(
    offramper: String,
    onramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid solana_backend principal".to_string()))?;

    let (res,): (SolanaResult<()>,) = call::<(String, String, u64, Option<String>), _>(
        can_id,
        "unlock_funds",
        (offramper, onramper, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}

pub async fn solana_backend_complete_order(
    onramper_address: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = Principal::from_text(SOLANA_BACKEND_CANISTER_ID)
        .map_err(|_| SystemError::InvalidInput("Invalid solana_backend principal".to_string()))?;

    let (res,): (SolanaResult<()>,) = call::<(String, u64, Option<String>), _>(
        can_id,
        "complete_order",
        (onramper_address, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}
