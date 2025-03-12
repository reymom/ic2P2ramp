use bitcoin::Txid;
use ic_btc_interface::Utxo;
use ic_cdk_timers::set_timer;
use std::{collections::HashMap, time::Duration};

use crate::{
    api, memory::stable::utxos::remove_rune_utxo_entries, Address, RuneID, RuneUTXOEntry,
    WalletConfig,
};

const TX_CHECK_INTERVAL: u64 = 10;
const MAX_TX_CHECKS: u8 = 20;

/// Monitors a rune transaction and cleans up spent UTXOs upon confirmation.
pub fn monitor_rune_transaction(
    config: WalletConfig,
    address: Address,
    tx_id: Txid,
    rune_id: RuneID,
    runes: HashMap<Utxo, RuneUTXOEntry>,
    attempt: u8,
) {
    if attempt >= MAX_TX_CHECKS {
        println!(
            "[monitor_transaction] Max attempts reached for tx {}",
            tx_id
        );
        return;
    }

    set_timer(Duration::from_secs(TX_CHECK_INTERVAL), move || {
        ic_cdk::spawn(async move {
            match api::bitcoin::get_utxos(config.network, config.btc_principal, address.to_string())
                .await
            {
                Ok(utxo_response) => {
                    // Check if the UTXOs from this tx_id are no longer present
                    let spent_utxos: Vec<Utxo> = runes
                        .keys()
                        .filter(|utxo| !utxo_response.contains(utxo))
                        .cloned()
                        .collect();

                    if !spent_utxos.is_empty() {
                        println!(
                            "[monitor_rune_transaction] Tx {} confirmed, removing spent UTXOs",
                            tx_id
                        );
                        let spent_entries: Vec<RuneUTXOEntry> = spent_utxos
                            .iter()
                            .filter_map(|utxo| runes.get(utxo).cloned())
                            .collect();
                        remove_rune_utxo_entries(&rune_id, spent_entries);
                    } else {
                        println!(
                            "[monitor_rune_transaction] Tx {} pending... Retrying...",
                            tx_id
                        );
                        monitor_rune_transaction(
                            config,
                            address,
                            tx_id,
                            rune_id,
                            runes,
                            attempt + 1,
                        );
                    }
                }
                Err(err) => {
                    println!("[monitor_rune_transaction] Error checking UTXOs: {:?}", err);
                }
            }
        });
    });
}
