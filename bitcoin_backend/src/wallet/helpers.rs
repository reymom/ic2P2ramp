use bitcoin::{
    absolute::LockTime, hashes::Hash, transaction::Version, Address, Amount, Network, OutPoint,
    ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Witness,
};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};

use crate::{
    api,
    model::types::errors::{BitcoinError, InsufficientBalanceError, Result},
    ordinals::runes::build_rune_script,
    TransactionType,
};

pub fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

pub fn build_transaction_with_fee(
    own_utxos: &[Utxo],
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    fee: u64,
    tx_type: TransactionType,
) -> Result<(Transaction, Vec<TxOut>)> {
    // Assume that any amount below this threshold is dust.
    const DUST_THRESHOLD: u64 = 1_000;

    // Select which UTXOs to spend. We naively spend the oldest available UTXOs,
    // even if they were previously spent in a transaction. This isn't a
    // problem as long as at most one transaction is created per block and
    // we're using min_confirmations of 1.

    // 1. Min number of utxos that sum amount (minimize current gas fees)
    // 2. Check historical fee and aggregate (consolidate) utxos when it is cheap

    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    for utxo in own_utxos.iter().rev() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        if total_spent >= amount + fee {
            // We have enough inputs to cover the amount we want to spend.
            break;
        }
    }

    if total_spent < amount + fee {
        return Err(BitcoinError::InsufficientBalance(
            InsufficientBalanceError {
                current_balance: total_spent,
                transfer_amount: amount,
                fee,
            },
        ));
    }

    let inputs: Vec<TxIn> = utxos_to_spend
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::MAX,
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        })
        .collect();

    let prevouts = utxos_to_spend
        .into_iter()
        .map(|utxo| TxOut {
            value: Amount::from_sat(utxo.value),
            script_pubkey: own_address.script_pubkey(),
        })
        .collect();

    let mut outputs = match tx_type {
        TransactionType::RuneTransfer(rune_id) => vec![TxOut {
            value: Amount::from_sat(0), // Runes do not require additional satoshis
            script_pubkey: build_rune_script(rune_id.parts().0, rune_id.parts().1.into()),
        }],
        _ => vec![TxOut {
            value: Amount::from_sat(amount),
            script_pubkey: dst_address.script_pubkey(),
        }],
    };

    let remaining_amount = total_spent - amount - fee;
    if remaining_amount >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: Amount::from_sat(remaining_amount),
        });
    }

    Ok((
        Transaction {
            input: inputs,
            output: outputs,
            lock_time: LockTime::ZERO,
            version: Version(2),
        },
        prevouts,
    ))
}

pub async fn get_fee_per_byte(network: BitcoinNetwork) -> Result<u64> {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = api::bitcoin::get_current_fee_percentiles(network).await?;

    if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        Ok(2000)
    } else {
        // Choose the 50th percentile for sending fees.
        Ok(fee_percentiles[50])
    }
}

// A mock for rubber-stamping signatures.
pub async fn mock_signer(
    _key_name: String,
    _derivation_path: Vec<Vec<u8>>,
    _message_hash: Vec<u8>,
) -> Vec<u8> {
    vec![255; 64]
}
