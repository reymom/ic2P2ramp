pub mod listeners;
pub mod memory;
pub mod model;
pub mod solana;
pub mod vault;

use candid::{Nat, Principal};
use ic_cdk::{init, post_upgrade, pre_upgrade, query, update};
use num_traits::cast::ToPrimitive;
use std::collections::HashMap;
use std::str::FromStr;

use sol_rpc_types::{GetAccountInfoEncoding, TokenAmount};
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_system_interface::instruction;
use solana_transaction::Transaction;

use crate::memory::stable::vault::{OFFRAMPER_VAULTS, ONRAMPER_VAULTS};
use crate::model::helpers::validate_caller_not_anonymous;
use crate::model::types::errors::SystemError;
use crate::model::types::tokens::fetch_mint_decimals;
use crate::model::types::{
    Address,
    errors::{Result, SolanaError, VaultError},
    tokens::validate_token_mint,
    vault::VaultEntry,
};
use crate::solana::client::client;
use crate::solana::spl;
use crate::solana::wallet::SolanaWallet;
use crate::{
    memory::heap::{
        InstallArg,
        config::{get_state, init_state},
        state::State,
        upgrade,
    },
    solana::account::get_account_owner,
};

#[pre_upgrade]
fn pre_upgrade() {
    upgrade::pre_upgrade()
}

#[post_upgrade]
fn post_upgrade(install_arg: InstallArg) {
    ic_cdk::println!(
        "[post_upgrade]: upgrade canister executed with install_arg: {:?}",
        install_arg
    );

    match install_arg {
        InstallArg::Reinstall(_) => ic_cdk::trap("InitArg not valid for reinstall"),
        InstallArg::Upgrade(update_arg) => {
            upgrade::post_upgrade(update_arg.clone());
        }
    }

    let state = get_state();
    ic_cdk::println!("[post_upgrade]: state = {:?}", state);
}

/// Called when canister is first installed.
#[init]
fn init(install_arg: InstallArg) {
    match install_arg {
        InstallArg::Reinstall(init_arg) => {
            let sol_rpc_canister_id = match init_arg.sol_rpc_canister_id {
                Some(canister_id) => canister_id,
                None => sol_rpc_client::SOL_RPC_CANISTER,
            };
            let state = State {
                sol_rpc_canister_id,
                network: init_arg.network,
                ed25519_key_name: init_arg.ed25519_key_name,
                ed25519_root_pk: None,
                proxy_url: init_arg.proxy_url,
            };

            ic_cdk::println!("[init]: state: {:?}", state);
            init_state(state);
        }
        InstallArg::Upgrade(_) => ic_cdk::trap("UpdateArg not valid for reinstall"),
    }
}

// ------
// Solana
// ------

// ----------------------------------------
// owned_* functions are just for reference
// ----------------------------------------

/// Derive a Solana address (Pubkey) from the canister’s threshold‐Ed25519 key.
/// If `caller` is `null`, use the caller's principal; otherwise use the provided one.
#[update]
async fn owned_solana_account(owner: Option<Principal>) -> String {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    wallet.solana_account().to_string()
}

#[update]
async fn owned_associated_token_account(
    owner: Option<Principal>,
    mint_account: String,
) -> Result<String> {
    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    let mint = Pubkey::from_str(&mint_account).map_err(SolanaError::from)?;
    Ok(spl::get_associated_token_address(
        wallet.solana_account().as_ref(),
        &mint,
        &get_account_owner(&mint).await?,
    )
    .to_string())
}

#[update]
async fn owned_get_spl_token_balance(
    account: Option<String>,
    mint_account: String,
) -> Result<TokenAmount> {
    let account = account.unwrap_or(owned_associated_token_account(None, mint_account).await?);
    let public_key = Pubkey::from_str(&account).map_err(SolanaError::from)?;
    Ok(client()
        .get_token_account_balance(public_key)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .into())
}

// ----------------------------------------
// canister_* functions are the useful ones
// ----------------------------------------

#[update]
async fn canister_solana_account() -> String {
    SolanaWallet::new_canister()
        .await
        .solana_account()
        .to_string()
}

// --------------------------------
// principal-agnostic balance calls
// --------------------------------

#[update]
async fn associated_token_account(account: String, mint_account: String) -> Result<String> {
    let public_key = Pubkey::from_str(&account).map_err(SolanaError::from)?;
    let mint = Pubkey::from_str(&mint_account).map_err(SolanaError::from)?;
    Ok(
        spl::get_associated_token_address(&public_key, &mint, &get_account_owner(&mint).await?)
            .to_string(),
    )
}

#[update]
async fn get_balance(account: String) -> Result<Nat> {
    let public_key = Pubkey::from_str(&account).map_err(SolanaError::from)?;
    let balance = client()
        .get_balance(public_key)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;
    Ok(Nat::from(balance))
}

#[update]
async fn get_spl_token_balance(account: String, mint_account: String) -> Result<TokenAmount> {
    let account = associated_token_account(account, mint_account).await?;
    let public_key = Pubkey::from_str(&account).map_err(SolanaError::from)?;
    Ok(client()
        .get_token_account_balance(public_key)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .into())
}

/// Creates the token account for a given mint for the specified principal
#[update]
async fn create_associated_token_account(
    owner: Option<Principal>,
    mint_account: String,
) -> Result<String> {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    let payer = wallet.solana_account();

    let mint = Pubkey::from_str(&mint_account).map_err(SolanaError::from)?;
    let (associated_token_account, instruction) = spl::create_associated_token_account_instruction(
        payer.as_ref(),
        payer.as_ref(),
        &mint,
        &get_account_owner(&mint).await?,
    );

    if let Some(_account) = client
        .get_account_info(associated_token_account)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
    {
        ic_cdk::println!(
            "[create_associated_token_account]: Account {} already exists. Skipping creation of associated token account",
            associated_token_account
        );
        return Ok(associated_token_account.to_string());
    }

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &client
            .estimate_recent_blockhash()
            .send()
            .await
            .map_err(|errors| {
                SolanaError::BlockhashErrors(format!(
                    "{:#?}",
                    errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                ))
            })?,
    );

    let signatures = vec![payer.sign_message(&message).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .to_string();

    Ok(associated_token_account.to_string())
}

/// Send raw SOL (lamports) to `dst_address`. Returns `Ok(txid)` on success.
#[update]
async fn send_sol(owner: Option<Principal>, dst: String, lamports: Nat) -> Result<String> {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;

    let recipient = Pubkey::from_str(&dst).map_err(SolanaError::from)?;
    let payer = wallet.solana_account();
    let amount = lamports
        .0
        .to_u64()
        .ok_or_else(|| SystemError::ParseError("lamports amount is not correct".to_string()))?;

    ic_cdk::println!(
        "Instruction to transfer {amount} lamports from {} to {recipient}",
        payer.as_ref()
    );
    let instruction = instruction::transfer(payer.as_ref(), &recipient, amount);

    let message = Message::new_with_blockhash(
        &[instruction],
        Some(payer.as_ref()),
        &client
            .estimate_recent_blockhash()
            .send()
            .await
            .map_err(|errors| {
                SolanaError::BlockhashErrors(format!(
                    "{:#?}",
                    errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                ))
            })?,
    );
    let signatures = vec![payer.sign_message(&message).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    let signature = client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;

    Ok(signature.to_string())
}

#[update]
async fn send_spl_token(
    owner: Option<Principal>,
    mint_account: String,
    to: String,
    amount: Nat,
) -> Result<String> {
    let client = client();

    let owner = owner.unwrap_or_else(validate_caller_not_anonymous);
    let wallet = SolanaWallet::new(owner).await;
    let payer = wallet.solana_account();

    let recipient = Pubkey::from_str(&to).map_err(SolanaError::from)?;
    let mint = Pubkey::from_str(&mint_account).map_err(SolanaError::from)?;
    let amount = amount
        .0
        .to_u64()
        .ok_or_else(|| SystemError::ParseError("lamports amount is not correct".to_string()))?;

    let token_program = get_account_owner(&mint).await?;

    let from = spl::get_associated_token_address(payer.as_ref(), &mint, &token_program);
    let to_ata = spl::get_associated_token_address(&recipient, &mint, &token_program);

    // 1. Ensure source ATA exists
    let from_info = client
        .get_account_info(from)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;
    if from_info.is_none() {
        return Err(SolanaError::RpcError(
            "Source token account does not exist".to_string(),
        ));
    }

    // 2. If destination ATA is missing, create it first (same tx)
    let mut instructions = Vec::new();
    let to_info = client
        .get_account_info(to_ata)
        .with_encoding(GetAccountInfoEncoding::Base64)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;

    if to_info.is_none() {
        let (derived, create_ix) = spl::create_associated_token_account_instruction(
            payer.as_ref(), // fee payer
            &recipient,     // owner of the ATA to be created
            &mint,
            &token_program,
        );
        // Sanity check (should match `to_ata`)
        if derived != to_ata {
            return Err(SolanaError::SystemError(SystemError::ParseError(
                "Derived ATA mismatch".to_string(),
            )));
        }
        instructions.push(create_ix);
    }

    // 3. Transfer
    instructions.push(spl::transfer_instruction_with_program_id(
        &from,
        &to_ata,
        payer.as_ref(), // authority = payer (owner of `from`)
        amount,
        &token_program,
    ));

    let message = Message::new_with_blockhash(
        &instructions,
        Some(payer.as_ref()),
        &client
            .estimate_recent_blockhash()
            .send()
            .await
            .map_err(|errors| {
                SolanaError::BlockhashErrors(format!(
                    "{:#?}",
                    errors
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                ))
            })?,
    );
    let signatures = vec![payer.sign_message(&message).await];
    let transaction = Transaction {
        message,
        signatures,
    };

    Ok(client
        .send_transaction(transaction)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .to_string())
}

#[update]
async fn withdraw_solana_fees(_destination_address: Address, _amount: u64) -> Result<String> {
    ic_cdk::trap("TO DO")
}

// ------
// Tokens
// ------

#[query]
fn get_registered_tokens() -> HashMap<String, u8> {
    memory::heap::config::get_tokens()
}

#[update]
async fn register_tokens(tokens: Vec<String>) -> Result<()> {
    let mut verified: HashMap<String, u8> = HashMap::with_capacity(tokens.len());

    for mint_str in tokens {
        validate_token_mint(mint_str.clone())?;

        let mint = Pubkey::from_str(&mint_str)
            .map_err(|e| SolanaError::SystemError(SystemError::ParseError(e.to_string())))?;

        let dec = fetch_mint_decimals(&mint).await?;
        verified.insert(mint_str, dec);
    }

    memory::heap::config::register_tokens(verified)
}

#[query]
fn is_token_supported(mint: String) -> Result<()> {
    validate_token_mint(mint.clone())?;
    if !crate::memory::heap::config::is_token_registered(mint.as_str()) {
        return Err(SolanaError::UnsupportedToken(mint.to_string()));
    };
    Ok(())
}

// -----
// VAULT
// -----

#[query]
fn get_offramper_deposits(offramper: Address) -> Result<VaultEntry> {
    OFFRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&offramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

#[query]
fn get_onramper_deposits(onramper: Address) -> Result<VaultEntry> {
    ONRAMPER_VAULTS
        .with_borrow(|vaults| vaults.get(&onramper))
        .ok_or_else(|| VaultError::AddressVaultNotFound.into())
}

/// Deposit lamports or an SPL‐token amount into the vault (offramper).
#[update]
fn deposit_to_vault_canister(
    offramper: String,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    if let Some(ref mint) = token_mint {
        if !crate::memory::heap::config::is_token_registered(mint) {
            return Err(SolanaError::UnsupportedToken(mint.to_string()));
        }
    }

    vault::deposit::deposit_to_vault(offramper, amount, token_mint.clone())?;

    Ok(())
}

/// Cancel (refund) a vault deposit.
#[update]
fn cancel_deposit(offramper: String, amount: u64, token_mint: Option<String>) -> Result<()> {
    if let Some(ref mint) = token_mint {
        if !crate::memory::heap::config::is_token_registered(mint) {
            return Err(SolanaError::UnsupportedToken(mint.to_string()));
        }
    }

    vault::deposit::cancel_deposit(offramper, amount, token_mint)?;

    Ok(())
}

#[update]
fn lock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    if let Some(ref mint) = token_mint {
        if !crate::memory::heap::config::is_token_registered(mint) {
            return Err(SolanaError::UnsupportedToken(mint.to_string()));
        }
    }

    vault::lock::lock_funds(offramper, onramper, amount, token_mint)
}

#[update]
fn unlock_funds(
    offramper: Address,
    onramper: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    if let Some(ref mint) = token_mint {
        if !crate::memory::heap::config::is_token_registered(mint) {
            return Err(SolanaError::UnsupportedToken(mint.to_string()));
        }
    }

    vault::lock::unlock_funds(offramper, onramper, amount, token_mint)
}

#[update]
async fn complete_order(
    onramper_address: Address,
    amount: u64,
    token_mint: Option<String>,
) -> Result<()> {
    if let Some(ref mint) = token_mint {
        if !crate::memory::heap::config::is_token_registered(mint) {
            return Err(SolanaError::UnsupportedToken(mint.to_string()));
        }
    }

    vault::complete::complete_order(onramper_address, amount, token_mint)?;

    Ok(())
}

ic_cdk::export_candid!();
