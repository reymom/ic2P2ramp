use candid::{Nat, Principal};
use ic_cdk::call;

use crate::model::{
    errors::{Result, SystemError},
    memory::heap::read_state,
};
use icramp_types::solana::{
    errors::Result as SolanaResult,
    fees::SolanaFeeEstimates,
    token::TokenInfo,
    transaction::{TxInfo, TxMetadata},
};

pub async fn solana_backend_solana_account() -> Result<String> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);
    let (account,): (String,) = (call::<(), (String,)>(can_id, "canister_solana_account", ())
        .await
        .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
        .0,);

    Ok(account)
}

pub async fn solana_backend_send_sol(dst: String, lamports: Nat) -> Result<String> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);
    let (tx_id,): (SolanaResult<String>,) = (call::<
        (Option<Principal>, String, Nat),
        (SolanaResult<String>,),
    >(
        can_id, "send_sol", (Some(can_id), dst, lamports)
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
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

    let (tx_id,): (SolanaResult<String>,) = (call::<
        (Option<Principal>, String, String, Nat),
        (SolanaResult<String>,),
    >(
        can_id,
        "send_spl_token",
        (Some(ic_cdk::id()), mint_account, to, amount),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?
    .0,);

    Ok(tx_id?)
}

pub async fn solana_backend_get_tx(signature: String) -> Result<TxInfo> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

    let (res,): (SolanaResult<TxInfo>,) = call::<(String,), _>(can_id, "get_tx", (signature,))
        .await
        .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}

pub async fn solana_backend_get_tx_metadata(signature: String) -> Result<TxMetadata> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);
    let (res,): (SolanaResult<TxMetadata>,) =
        call::<(String,), _>(can_id, "get_tx_metadata", (signature,))
            .await
            .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;
    Ok(res?)
}

pub async fn solana_backend_estimate_fees(mint: Option<String>) -> Result<SolanaFeeEstimates> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);
    let (res,): (SolanaResult<SolanaFeeEstimates>,) =
        call::<(Option<String>,), _>(can_id, "estimate_fees", (mint,))
            .await
            .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;
    Ok(res?)
}

pub async fn solana_backend_get_token_info(mint: String) -> Result<TokenInfo> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);
    let (res,): (SolanaResult<TokenInfo>,) =
        call::<(String,), _>(can_id, "get_token_info", (mint,))
            .await
            .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;
    Ok(res?)
}

pub async fn solana_backend_deposit_funds(
    offramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

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
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

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
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

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
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

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
    let can_id = read_state(|s| s.canister_ids.solana_backend_id);

    let (res,): (SolanaResult<()>,) = call::<(String, u64, Option<String>), _>(
        can_id,
        "complete_order",
        (onramper_address, amount, token_mint),
    )
    .await
    .map_err(|(code, err)| SystemError::ICRejectionError(code, err))?;

    Ok(res?)
}
