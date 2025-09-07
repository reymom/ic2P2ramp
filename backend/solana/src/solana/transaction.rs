use base64::{Engine as _, engine::general_purpose::STANDARD};
use bincode::{config::standard, serde::decode_from_slice};
use icramp_types::solana::{
    errors::{Result, SolanaError, TransactionError},
    transaction::{TxInfo, TxMetadata},
};
use sol_rpc_types::{GetTransactionEncoding, TransactionStatusMeta};
use solana_signature::Signature;
use solana_transaction::versioned::VersionedTransaction;
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, EncodedTransaction, UiLoadedAddresses,
    UiTransactionStatusMeta,
};
use std::str::FromStr;

use crate::solana::client::client;

pub async fn get_tx(signature_b58: String) -> Result<TxInfo> {
    let (confirmed, confirmations, confirmation_status, slot, err) =
        fetch_status(&signature_b58).await?;

    Ok(TxInfo {
        signature: signature_b58,
        found: true,
        confirmed,
        confirmations,
        confirmation_status,
        slot,
        err,
    })
}

pub async fn get_tx_metadata(signature_b58: String) -> Result<TxMetadata> {
    let (confirmed, _confs, _status, _slot, err) = fetch_status(&signature_b58).await?;
    if err.is_some() {
        return Err(TransactionError::MetaError(format!("tx failed: {:?}", err)).into());
    }
    if !confirmed {
        return Err(TransactionError::Unconfirmed(signature_b58.clone()).into());
    }

    let sig = Signature::from_str(&signature_b58)
        .map_err(|e| SolanaError::ParsePubkeyError(e.to_string()))?;

    // Detailed fetch (meta + encoded tx) only after confirmed
    let EncodedConfirmedTransactionWithStatusMeta {
        slot,
        transaction,
        block_time: _,
    } = client()
        .get_transaction(sig)
        .with_encoding(GetTransactionEncoding::Base64)
        .with_max_supported_transaction_version(0)
        .with_cycles(4_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .ok_or_else(|| {
            TransactionError::MetaError("encoded confirmation transaction not found".to_string())
        })?;

    ic_cdk::println!(
        "[get_tx_metadata] slot: {}, transaction: {:?}",
        slot,
        transaction
    );
    let ui_meta: UiTransactionStatusMeta = transaction
        .meta
        .clone()
        .ok_or_else(|| TransactionError::MetaError("missing meta".into()))?;

    let meta: TransactionStatusMeta = ui_meta
        .try_into()
        .map_err(|e| TransactionError::MetaError(format!("meta convert: {e:?}")))?;

    // --- ACCOUNT KEYS (decode Base64 once) ---
    let tx_b64 = match &transaction.transaction {
        EncodedTransaction::LegacyBinary(s) => s.as_str(),
        EncodedTransaction::Binary(s, _) => s.as_str(),
        _ => return Err(TransactionError::MetaError("unsupported tx encoding".into()).into()),
    };
    let tx_bytes = STANDARD
        .decode(tx_b64)
        .map_err(|e| TransactionError::MetaError(format!("base64 decode: {e}")))?;
    let (vtx, _): (VersionedTransaction, usize) = decode_from_slice(&tx_bytes, standard())
        .map_err(|e| TransactionError::MetaError(format!("bincode v2 decode: {e}")))?;

    let mut account_keys: Vec<String> = vtx
        .message
        .static_account_keys()
        .iter()
        .map(|k| k.to_string())
        .collect();

    // append loaded addresses (writable then readonly)
    if let Some(la) = transaction.meta.as_ref().and_then(|m| {
        let os = m.loaded_addresses.as_ref();
        Into::<Option<&UiLoadedAddresses>>::into(os)
    }) {
        for k in &la.writable {
            account_keys.push(k.clone());
        }
        for k in &la.readonly {
            account_keys.push(k.clone());
        }
    }

    ic_cdk::println!("[get_tx_metadata] account keys = {:?}", account_keys);

    Ok(TxMetadata {
        signature: signature_b58,
        slot,
        meta,
        account_keys,
    })
}

/// INTERNAL: shared status fetch/normalize
async fn fetch_status(
    signature_b58: &str,
) -> Result<(
    bool,           // confirmed
    Option<u64>,    // confirmations
    Option<String>, // status
    Option<u64>,    // slot
    Option<String>, // error
)> {
    let sig = Signature::from_str(signature_b58)
        .map_err(|e| SolanaError::ParsePubkeyError(e.to_string()))?;

    let status = client()
        .get_signature_statuses([&sig])?
        .with_search_transaction_history(true)
        .with_cycles(3_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .into_iter()
        .next()
        .flatten();

    ic_cdk::println!("[fetch_status], status: {:?}", status);

    if let Some(s) = status {
        let confs = s.confirmations.map(|c| c as u64);
        let slot = Some(s.slot);
        let status_str = s
            .confirmation_status
            .as_ref()
            .map(|cs| format!("{cs:?}").to_lowercase());

        let confirmed = matches!(status_str.as_deref(), Some("confirmed") | Some("finalized"))
            || s.confirmations.unwrap_or(0) > 0;
        let err = s.err.as_ref().map(|e| e.to_string());
        Ok((confirmed, confs, status_str, slot, err))
    } else {
        Err(TransactionError::NotFound(signature_b58.to_string()).into())
    }
}
