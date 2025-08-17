use icramp_types::solana::{
    errors::{Result, SolanaError},
    transaction::TxInfo,
};
use solana_signature::Signature;
use std::str::FromStr;

use crate::solana::client::client;

pub async fn get_tx(signature_b58: String) -> Result<TxInfo> {
    let sig = Signature::from_str(&signature_b58)
        .map_err(|e| SolanaError::ParsePubkeyError(e.to_string()))?;

    // Query the status (cheap + enough for confirmation)
    let statuses = client()
        .get_signature_statuses([&sig])?
        .with_cycles(3_000_000_000)
        .send()
        .await
        .expect_consistent()
        .map_err(SolanaError::from)?;

    let status_opt = statuses.into_iter().next().flatten();

    // Default (not found yet)
    let mut info = TxInfo {
        signature: signature_b58,
        found: false,
        confirmed: false,
        confirmations: None,
        confirmation_status: None,
        slot: None,
        err: None,
    };

    if let Some(s) = status_opt {
        info.found = true;
        info.confirmations = s.confirmations.map(|c| c as u64);
        info.slot = Some(s.slot);
        let status_str = s
            .confirmation_status
            .as_ref()
            .map(|cs| format!("{cs:?}").to_lowercase());
        info.confirmation_status = status_str.clone();
        info.confirmed = matches!(status_str.as_deref(), Some("confirmed") | Some("finalized"))
            || s.confirmations.unwrap_or(0) > 0;
        info.err = s.err.as_ref().map(|e| e.to_string());
    }

    Ok(info)
}
