use bitcoin::{consensus, ScriptBuf, Transaction};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, GetUtxosResponse, Utxo};

use crate::errors::{BitcoinError, Result};

/// Fetches runes from the outputs associated with a list of UTXOs.
pub async fn fetch_runes_from_utxos(
    network: BitcoinNetwork,
    utxos: Vec<Utxo>,
) -> Result<Vec<(String, u64)>> {
    let mut runes = Vec::new();

    for utxo in utxos {
        // Fetch the raw transaction using the UTXO's outpoint (txid and vout).
        let tx_bytes = fetch_raw_transaction(network, &utxo.outpoint.txid).await?;
        let transaction: Transaction = consensus::deserialize(&tx_bytes)
            .map_err(|e| BitcoinError::InternalError(e.to_string()))?;

        // Check the output script of the UTXO.
        if let Some(tx_out) = transaction.output.get(utxo.outpoint.vout as usize) {
            if let Some(rune) = extract_rune_from_script(&tx_out.script_pubkey) {
                runes.push((rune, tx_out.value.to_sat())); // Add the rune symbol and amount.
            }
        }
    }

    Ok(runes)
}

/// Fetches a raw transaction by txid from the Bitcoin network.
async fn fetch_raw_transaction(network: BitcoinNetwork, txid: &[u8]) -> Result<Vec<u8>> {
    let txid_str = hex::encode(txid);
    let request = GetTransactionRequest {
        txid: txid_str.clone(),
        network,
    };
    let (response,) = bitcoin::bitcoin_get_transaction(request)
        .await
        .map_err(|e| format!("Failed to fetch tx {}: {:?}", txid_str, e))?;
    Ok(response.transaction)
}

/// Extracts rune information from a given script_pubkey.
fn extract_rune_from_script(script: &ScriptBuf) -> Option<String> {
    // Example: Parse OP_RETURN data for rune information.
    if script.is_op_return() {
        if let Some(data) = script.iter_push().next() {
            return String::from_utf8(data.to_vec()).ok();
        }
    }
    None
}
