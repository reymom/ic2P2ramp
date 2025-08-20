use icramp_types::solana::{
    errors::{Result, SolanaError, TransactionError},
    transaction::{TxInfo, TxMetadata},
};
use sol_rpc_types::TransactionStatusMeta;
use solana_signature::Signature;
use solana_transaction_status_client_types::{
    EncodedConfirmedTransactionWithStatusMeta, UiTransactionStatusMeta,
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
    // 2) Detailed fetch (meta + encoded tx) only after confirmed
    let EncodedConfirmedTransactionWithStatusMeta {
        slot,
        transaction,
        block_time: _,
    } = client()
        .get_transaction(sig)
        .with_cycles(4_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .ok_or_else(|| {
            TransactionError::MetaError("encoded confirmation transaction not found".to_string())
        })?;

    let ui_meta: UiTransactionStatusMeta = transaction
        .meta
        .ok_or_else(|| TransactionError::MetaError("missing meta".into()))?;

    let meta: TransactionStatusMeta = ui_meta
        .try_into()
        .map_err(|e| TransactionError::MetaError(format!("meta convert: {e:?}")))?;

    Ok(TxMetadata {
        signature: signature_b58,
        slot,
        meta,
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
        .with_cycles(3_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?
        .into_iter()
        .next()
        .flatten();

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
