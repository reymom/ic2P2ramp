use std::collections::HashMap;

use evm_rpc_canister_types::BlockTag;
use icramp_types::bitcoin::runes::RuneID;
use icramp_types::solana::errors::{SolanaError, TransactionError};

use crate::evm::event::LogEvent;
use crate::evm::fees::eth_get_latest_block;
use crate::inter_canister::bitcoin::bitcoin_backend_validate_rune;
use crate::inter_canister::solana::{
    solana_backend_get_tx_metadata, solana_backend_solana_account,
};
use crate::management::order::get_valid_log_event;
use crate::model::errors::{BlockchainError, OrderError, Result};
use crate::model::memory::stable::spent_transactions;
use crate::model::types::evm::{chains, token};
use crate::model::types::orders::DepositInput;

pub async fn verify_evm_transaction(
    chain_id: u64,
    token_address: Option<String>,
    deposit_input: Option<DepositInput>,
    sender: &str,
    // receiver: &str,
    amount: u128,
) -> Result<Option<String>> {
    chains::chain_is_supported(chain_id)?;
    if let Some(token) = token_address.clone() {
        token::evm_token_is_approved(chain_id, &token)?;
    };

    let evm_input = match deposit_input {
        Some(DepositInput::Evm(v)) => Ok(v),
        _ => Err(OrderError::InvalidInput(
            "Missing evm order input".to_string(),
        )),
    }?;

    let tx_hash = evm_input.tx_hash.clone();

    let (log_event, log_entry) = get_valid_log_event(&chain_id, &tx_hash).await?;
    ic_cdk::println!(
        "[verify_crypto_transaction][evm] log_event = {:?}",
        log_event
    );

    match log_event {
        LogEvent::Deposit(deposit_event) => {
            if deposit_event.user.to_lowercase() != sender.to_lowercase() {
                return Err(
                    BlockchainError::EvmLogError("Invalid Offramper Address".to_string()).into(),
                );
            }
            if deposit_event.amount != amount {
                return Err(
                    BlockchainError::EvmLogError("Invalid Crypto Amount".to_string()).into(),
                );
            }
            if deposit_event.token.clone().map(|t| t.to_lowercase())
                != token_address.clone().map(|t| t.to_lowercase())
            {
                return Err(BlockchainError::EvmLogError("Invalid Crypto".to_string()).into());
            }

            // log_entry.address is token contract emitting the Deposit, which is in turn the receiver
            // if log_entry.address != receiver {
            //     return Err(BlockchainError::EvmLogError("Invalid Receiver".to_string()).into());
            // }

            let last_block = eth_get_latest_block(chain_id, BlockTag::Latest)
                .await
                .map(|block| block.number)?;
            deposit_event.expired(last_block)?;
        }

        LogEvent::Transfer(ev) => {
            // The pay-with-crypto case.
            // Here sender = **onramper provider address**
            // Destination = offramper provider address which you passed into verify

            if ev.from.to_lowercase() != sender.to_lowercase() {
                return Err(BlockchainError::EvmLogError("Invalid sender".into()).into());
            }
            // if ev.to.to_lowercase() != receiver.to_lowercase() {
            //     return Err(BlockchainError::EvmLogError("Invalid receiver".into()).into());
            // }

            // For token case, ev.value is guaranteed correct
            if ev.value != amount {
                return Err(BlockchainError::EvmLogError("Invalid amount".into()).into());
            }

            // Token must match asset token
            if let Some(token) = token_address.clone() {
                // log_entry.address is token contract emitting the Transfer
                if log_entry.address.to_lowercase() != token.to_lowercase() {
                    return Err(
                        BlockchainError::EvmLogError("Invalid token contract".into()).into(),
                    );
                }
            }
        }
    };

    Ok(Some(tx_hash))
}

pub async fn verify_bitcoin_deposit(
    maybe_rune_id: Option<RuneID>,
    deposit_input: Option<DepositInput>,
    amount: u128,
) -> Result<Option<String>> {
    if let Some(rune_id) = maybe_rune_id.clone() {
        bitcoin_backend_validate_rune(rune_id).await?;
    };

    let bitcoin_input = match deposit_input {
        Some(DepositInput::Bitcoin(v)) => Ok(v),
        _ => Err(OrderError::InvalidInput(
            "Missing bitcoin order input".to_string(),
        )),
    }?;

    let bitcoin_txid = bitcoin_input.tx_id;

    if spent_transactions::is_tx_hash_processed(&bitcoin_txid) {
        return Err(BlockchainError::BitcoinBackendError(
            "Transaction already processed".to_string(),
        )
        .into());
    };

    // For BTC we only ensure non-duplication here; actual confirmation + UTXO
    // validation is handled by the bitcoin listener before creating the order.
    if amount == 0 {
        return Err(OrderError::InvalidInput("Order amount is zero".to_string()).into());
    }

    Ok(Some(bitcoin_txid))
}

pub async fn verify_solana_deposit(
    expected_mint: Option<String>,
    deposit_input: Option<DepositInput>,
    amount: u128,
) -> Result<Option<String>> {
    let sol_input = match deposit_input {
        Some(DepositInput::Solana(v)) => Ok(v),
        _ => Err(OrderError::InvalidInput(
            "Missing solana order input".to_string(),
        )),
    }?;
    let vault_addr = solana_backend_solana_account().await?;

    if spent_transactions::is_tx_hash_processed(&sol_input.signature) {
        return Err(BlockchainError::TransactionAlreadyProcessed.into());
    }

    // Optional local input sanity vs asset
    match (&expected_mint, &sol_input.mint) {
        (Some(exp), Some(got)) if exp != got => {
            return Err(OrderError::InvalidInput("SPL mint mismatch".to_string()).into());
        }
        (None, Some(_)) => {
            return Err(OrderError::InvalidInput(
                "Unexpected SPL mint for SOL deposit".to_string(),
            )
            .into());
        }
        _ => {}
    }

    let txm = solana_backend_get_tx_metadata(sol_input.signature.clone()).await?;
    println!("[verify_crypto_transaction][solana] TxMetadata: {:?}", txm);
    if !txm.meta.status.is_ok() {
        return Err(
            SolanaError::from(TransactionError::MetaError("tx meta status != Ok".into())).into(),
        );
    }

    let keys = &txm.account_keys;
    let pre = &txm.meta.pre_balances;
    let post = &txm.meta.post_balances;
    if pre.len() != post.len() || pre.len() != keys.len() {
        return Err(SolanaError::from(TransactionError::MetaError(
            "invalid lamport vectors / keys".into(),
        ))
        .into());
    }

    if let Some(exp_mint) = expected_mint {
        // -------- SPL TOKEN VALIDATION (credit must go to vault) --------
        let mut pre_map: HashMap<u8, u128> = HashMap::new();
        if let Some(pre_tbs) = txm.meta.pre_token_balances.as_ref() {
            for tb in pre_tbs {
                if tb.mint == exp_mint {
                    if let (i, Some(a)) = (
                        tb.account_index,
                        tb.ui_token_amount.amount.parse::<u128>().ok(),
                    ) {
                        pre_map.insert(i, a);
                    }
                }
            }
        }

        let mut hits = 0usize;
        if let Some(post_tbs) = txm.meta.post_token_balances.as_ref() {
            for tb in post_tbs {
                if tb.mint == exp_mint {
                    if let (i, Some(post_amt)) = (
                        tb.account_index,
                        tb.ui_token_amount.amount.parse::<u128>().ok(),
                    ) {
                        let pre_amt = *pre_map.get(&i).unwrap_or(&0);
                        let delta = post_amt.saturating_sub(pre_amt);
                        println!("[verify_crypto_transaction][solana] spl delta: {}", delta);
                        if delta > 0 {
                            let owner_ok = tb
                                .owner
                                .as_ref()
                                .map(|o| o.to_string() == vault_addr)
                                .unwrap_or(false);
                            if !owner_ok {
                                return Err(SolanaError::from(TransactionError::MetaError(
                                    "SPL credit not to vault".into(),
                                ))
                                .into());
                            }
                            if delta == amount {
                                hits += 1;
                            } else {
                                return Err(SolanaError::from(TransactionError::MetaError(
                                    "ambiguous SPL credits in tx".into(),
                                ))
                                .into());
                            }
                        }
                    }
                }
            }
        }
        if hits != 1 {
            return Err(SolanaError::from(TransactionError::MetaError(
                "SPL deposit not found / ambiguous".into(),
            ))
            .into());
        }
    } else {
        // -------- SOL (lamports) VALIDATION with address binding --------
        let mut hits = 0usize;
        for ((a, b), addr) in pre.iter().zip(post.iter()).zip(keys.iter()) {
            let delta = b.saturating_sub(*a) as u128;
            if delta > 0 {
                if addr != &vault_addr {
                    return Err(SolanaError::from(TransactionError::MetaError(
                        "lamports credit not to solana canister vault".into(),
                    ))
                    .into());
                }
                if delta == amount {
                    hits += 1;
                } else {
                    return Err(SolanaError::from(TransactionError::MetaError(
                        "ambiguous SOL credits in tx".into(),
                    ))
                    .into());
                }
            }
        }
        if hits != 1 {
            return Err(SolanaError::from(TransactionError::MetaError(
                "SOL deposit not found / ambiguous".into(),
            ))
            .into());
        }
    }

    if amount > u64::MAX as u128 {
        return Err(OrderError::InvalidInput("order amount too large".into()).into());
    }

    Ok(Some(sol_input.signature))
}
