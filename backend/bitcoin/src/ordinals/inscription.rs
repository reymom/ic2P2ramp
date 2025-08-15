use bitcoin::{
    opcodes,
    script::{Builder, PushBytesBuf},
    ScriptBuf, Txid,
};
use ic_btc_interface::Satoshi;

use crate::model::types::{
    errors::{BitcoinError, Result},
    inscription::Inscription,
    transfer::TransactionType,
    wallet::WalletConfig,
};

pub fn build_ordinal_inscription(inscription: &Inscription) -> Result<ScriptBuf> {
    let inscription_data = format!(
        "{}\n{}\n{}",
        inscription.content,
        inscription.content_type,
        inscription.metadata.clone().unwrap_or_default()
    )
    .into_bytes();

    if inscription_data.len() > 80 {
        return Err(BitcoinError::InvalidRunestone(
            "Ordinal inscription data is too large".to_string(),
        ));
    }

    let data_pushbytes = PushBytesBuf::try_from(inscription_data).map_err(|_| {
        BitcoinError::InternalError("Failed to encode inscription data".to_string())
    })?;

    Ok(Builder::new()
        .push_opcode(opcodes::all::OP_RETURN)
        .push_slice(&data_pushbytes)
        .into_script())
}

pub async fn inscribe_ordinal(
    config: WalletConfig,
    dst_address: String,
    amount: Satoshi,
    inscription: Inscription,
) -> Result<Txid> {
    crate::wallet::p2tr_script_spend::send_script_spend(
        config,
        dst_address,
        amount,
        TransactionType::OrdinalInscription(inscription),
    )
    .await
}
