use hex::FromHex;
use std::collections::HashMap;

use ic_btc_interface::Utxo;
use icramp_types::bitcoin::errors::Result;

use crate::{
    RuneUTXOEntry, WalletConfig,
    api::{self, unisat::fetch_rune_utxos},
    get_registered_runes,
};

pub async fn get_tx_utxos(
    config: WalletConfig,
    address: String,
    rune_utxos: Option<Vec<RuneUTXOEntry>>,
) -> Result<(Vec<Utxo>, HashMap<Utxo, RuneUTXOEntry>)> {
    let all_utxos =
        api::bitcoin::get_utxos(config.network, config.btc_principal, address.to_string()).await?;
    let all_rune_utxos: Vec<(String, u32)> = fetch_all_rune_utxos(&address)
        .await?
        .iter()
        .map(|utxo| (utxo.txid.clone(), utxo.vout))
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
                    .as_ref()
                    .iter()
                    .rev()
                    .cloned()
                    .collect::<Vec<u8>>(),
            );
            !all_rune_utxos.contains(&(utxo_txid_be, utxo.outpoint.vout))
        })
        .collect();

    let rune_utxo_map: HashMap<Utxo, RuneUTXOEntry> = if let Some(utxos) = rune_utxos {
        utxos
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
                                .as_ref()
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

    Ok((btc_utxos, rune_utxo_map))
}

pub async fn fetch_all_rune_utxos(address: &str) -> Result<Vec<RuneUTXOEntry>> {
    let registered_runes = get_registered_runes()?;

    let mut all_utxos = Vec::new();
    for rune in registered_runes {
        match fetch_rune_utxos(address, rune.id.clone()).await {
            Ok(mut utxos) => {
                all_utxos.append(&mut utxos);
            }
            Err(e) => {
                ic_cdk::println!("Error fetching UTXOs for rune {}: {:?}", rune.id, e);
                return Err(e);
            }
        }
    }

    Ok(all_utxos)
}
