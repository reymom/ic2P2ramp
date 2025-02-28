use hex::FromHex;
use std::collections::HashMap;

use ic_cdk::api::management_canister::bitcoin::Utxo;

use crate::{
    api,
    memory::stable::utxos::{get_rune_utxos, list_rune_utxos},
    types::errors::Result,
    RuneUTXOEntry, TransactionType, WalletConfig,
};

pub async fn get_tx_utxos(
    config: WalletConfig,
    address: String,
    tx_type: TransactionType,
) -> Result<(Vec<Utxo>, HashMap<Utxo, RuneUTXOEntry>)> {
    let all_utxos = api::bitcoin::get_utxos(config.network, address.to_string()).await?;
    let all_rune_utxos: Vec<(String, u32)> = list_rune_utxos()
        .iter()
        .map(|(_, utxo)| (utxo.txid.clone(), utxo.vout))
        .collect();
    // Filter out Rune UTXOs from the general UTXO list (ensuring only BTC UTXOs are left for fee payment)
    let btc_utxos: Vec<Utxo> = all_utxos
        .clone()
        .into_iter()
        .filter(|utxo| {
            // Reverse bytes to convert LE to BE
            let utxo_txid_be = hex::encode(
                utxo.outpoint
                    .txid
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<u8>>(),
            );
            !all_rune_utxos.contains(&(utxo_txid_be, utxo.outpoint.vout))
        })
        .collect();

    let rune_utxos: HashMap<Utxo, RuneUTXOEntry> =
        if let TransactionType::RuneTransfer(rune_id) = &tx_type {
            get_rune_utxos(&rune_id)
                .into_iter()
                .filter_map(|entry| {
                    all_utxos
                        .iter()
                        .find(|utxo| {
                            if let Ok(decoded_txid) = Vec::from_hex(&entry.txid) {
                                // Reverse bytes to convert LE to BE
                                let utxo_txid_be = utxo
                                    .outpoint
                                    .txid
                                    .iter()
                                    .rev()
                                    .cloned()
                                    .collect::<Vec<u8>>();
                                decoded_txid == utxo_txid_be && entry.vout == utxo.outpoint.vout
                            } else {
                                false
                            }
                        })
                        .cloned()
                        .map(|utxo| (utxo, entry))
                })
                .collect()
        } else {
            HashMap::new()
        };

    Ok((btc_utxos, rune_utxos))
}
