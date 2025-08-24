use std::collections::HashMap;

use bitcoin::{
    Address, Amount, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness, absolute::LockTime, hashes::Hash, transaction::Version,
};
use candid::Principal;
use ic_btc_interface::{Network as BitcoinNetwork, Utxo};
use icramp_types::bitcoin::errors::{BitcoinError, InsufficientBalanceError, Result};

use crate::{
    RuneID, RuneUTXOEntry, TransactionType, api,
    model::types::transfer::TaprootUseCase,
    ordinals::{
        inscription::build_ordinal_inscription,
        runes::{build_runestone_edict, build_runestone_etching},
    },
};

const DUST_THRESHOLD: u64 = 546;

pub fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

pub fn build_transaction_with_fee(
    tx_type: TransactionType,
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    fee: u64,
    btc_utxos: &[Utxo],
    rune_utxos: Option<HashMap<Utxo, RuneUTXOEntry>>,
) -> Result<(Transaction, Vec<TxOut>)> {
    let mut inputs = Vec::new();
    let mut prevouts = Vec::new();
    let mut outputs = Vec::new();

    match tx_type {
        TransactionType::RuneTransfer(rune_id) => {
            // 1. Handle rune UTXOs
            let (r_inputs, r_prevouts, r_outputs) = if let Some(rune_utxos) = rune_utxos {
                build_rune_inputs_and_outputs(
                    rune_utxos,
                    own_address,
                    dst_address,
                    amount,
                    rune_id,
                )?
            } else {
                return Err(BitcoinError::InternalError(
                    "Missing rune UTXOs".to_string(),
                ));
            };

            let required_dust: u64 = r_outputs
                .iter()
                .filter(|output| output.value.to_sat() > 0)
                .map(|output| output.value.to_sat())
                .sum();

            // 2. Handle BTC UTXOs for fee
            let (btc_inputs, btc_prevouts, btc_outputs) =
                build_btc_inputs_and_outputs(btc_utxos, own_address, None, required_dust, fee)?;

            // 3. Add all inputs, prevouts and outputs
            inputs.extend(r_inputs);
            inputs.extend(btc_inputs);

            prevouts.extend(r_prevouts);
            prevouts.extend(btc_prevouts);

            outputs.extend(r_outputs);
            outputs.extend(btc_outputs);
        }
        TransactionType::RuneEtching(_) | TransactionType::OrdinalInscription(_) => {
            // ✅ Handle BTC UTXOs for fee
            let (btc_inputs, btc_prevouts, btc_outputs) =
                build_btc_inputs_and_outputs(btc_utxos, own_address, None, 0, fee)?;

            // ✅ Construct OP_RETURN output for Runestone or Inscription
            let script_pubkey =
                if let Some(TaprootUseCase::RuneEtching(etching)) = tx_type.to_taproot_use_case() {
                    build_runestone_etching(etching)?
                } else if let Some(TaprootUseCase::Inscription(inscription)) =
                    tx_type.to_taproot_use_case()
                {
                    build_ordinal_inscription(&inscription)?
                } else {
                    return Err(BitcoinError::InternalError(
                        "Invalid taproot use case".to_string(),
                    ));
                };

            outputs.push(TxOut {
                value: Amount::from_sat(0),
                script_pubkey,
            });

            // ✅ Add destination output
            outputs.push(TxOut {
                value: Amount::from_sat(amount),
                script_pubkey: dst_address.script_pubkey(),
            });

            inputs.extend(btc_inputs);
            prevouts.extend(btc_prevouts);
            outputs.extend(btc_outputs);
        }
        _ => {
            return build_btc_transaction_with_fee(btc_utxos, own_address, dst_address, amount, fee);
        }
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

fn build_rune_inputs_and_outputs(
    rune_utxos: HashMap<Utxo, RuneUTXOEntry>,
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    rune_id: RuneID,
) -> Result<(Vec<TxIn>, Vec<TxOut>, Vec<TxOut>)> {
    let mut total_runes = 0;
    let mut r_inputs = Vec::new();
    let mut r_prevouts = Vec::new();
    let mut r_outputs = Vec::new();

    // 1: Construct Inputs
    for (utxo, rune_entry) in rune_utxos {
        total_runes += rune_entry.rune_amount;

        r_inputs.push(TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(utxo.outpoint.txid.as_ref()).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        });

        let script_pubkey = ScriptBuf::from_hex(&rune_entry.script_pubkey)
            .map_err(|_| BitcoinError::InternalError("Invalid script_pubkey".to_string()))?;
        r_prevouts.push(TxOut {
            value: Amount::from_sat(utxo.value),
            script_pubkey,
        });

        if total_runes >= amount {
            break;
        }
    }

    if total_runes < amount {
        return Err(BitcoinError::InsufficientBalance(
            InsufficientBalanceError {
                current_balance: total_runes,
                transfer_amount: amount,
                fee: 0,
            },
        ));
    }

    let rune_change = total_runes.saturating_sub(amount);
    let output_index: u32 = if rune_change > 0 { 2 } else { 1 };

    // 1: OP_RETURN (Runestone) Output (vout[0])
    let runestone = build_runestone_edict(&rune_id, amount, output_index)?;
    if runestone.len() > 82 {
        return Err(BitcoinError::InvalidRunestone(
            "Exceeds OP_RETURN size of 82".to_string(),
        ));
    }
    r_outputs.push(TxOut {
        value: Amount::from_sat(0),
        script_pubkey: runestone,
    });

    // 2: Sender's Change Output (vout[1])
    if rune_change > 0 {
        r_outputs.push(TxOut {
            value: Amount::from_sat(DUST_THRESHOLD),
            script_pubkey: own_address.script_pubkey(),
        });
    }

    // 3: Receiver's Output (vout[output_index])
    r_outputs.push(TxOut {
        value: Amount::from_sat(DUST_THRESHOLD),
        script_pubkey: dst_address.script_pubkey(),
    });

    Ok((r_inputs, r_prevouts, r_outputs))
}

fn build_btc_inputs_and_outputs(
    btc_utxos: &[Utxo],
    own_address: &Address,
    dst_address: Option<&Address>,
    amount: u64,
    fee: u64,
) -> Result<(Vec<TxIn>, Vec<TxOut>, Vec<TxOut>)> {
    // Select which UTXOs to spend. We naively spend the oldest available UTXOs,
    // even if they were previously spent in a transaction. This isn't a
    // problem as long as at most one transaction is created per block and
    // we're using min_confirmations of 1.

    // 1. Min number of utxos that sum amount (minimize current gas fees)
    // 2. Check historical fee and aggregate (consolidate) utxos when it is cheap

    let mut inputs = Vec::new();
    let mut prevouts = Vec::new();
    let mut outputs = Vec::new();

    let mut total_btc = 0;
    for utxo in btc_utxos.iter() {
        total_btc += utxo.value;
        inputs.push(TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid.as_ref()).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::MAX,
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        });
        prevouts.push(TxOut {
            value: Amount::from_sat(utxo.value),
            script_pubkey: own_address.script_pubkey(),
        });

        if total_btc >= amount + fee {
            break;
        }
    }

    if total_btc < amount + fee {
        return Err(BitcoinError::InsufficientBalance(
            InsufficientBalanceError {
                current_balance: total_btc,
                transfer_amount: amount,
                fee,
            },
        ));
    }

    if let Some(dst) = dst_address {
        if amount == 0 {
            return Err(BitcoinError::InvalidInput(
                "Amount cannot be zero".to_string(),
            ));
        }
        outputs.push(TxOut {
            value: Amount::from_sat(amount),
            script_pubkey: dst.script_pubkey(),
        });
    }

    let btc_change = total_btc - amount - fee;
    if btc_change >= DUST_THRESHOLD {
        outputs.push(TxOut {
            value: Amount::from_sat(btc_change),
            script_pubkey: own_address.script_pubkey(),
        });
    }

    Ok((inputs, prevouts, outputs))
}

fn build_btc_transaction_with_fee(
    own_utxos: &[Utxo],
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    fee: u64,
) -> Result<(Transaction, Vec<TxOut>)> {
    let (inputs, prevouts, outputs) =
        build_btc_inputs_and_outputs(own_utxos, own_address, Some(dst_address), amount, fee)?;

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

pub async fn get_fee_per_byte(network: BitcoinNetwork, btc_principal: Principal) -> Result<u64> {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = api::bitcoin::get_current_fee_percentiles(network, btc_principal).await?;

    if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        return Ok(2000);
    }

    let median_index = fee_percentiles.len() / 2;
    let fee_per_msat = fee_percentiles[median_index];

    ic_cdk::println!(
        "[get_fee_per_byte] Fee percentiles: {:?}, selected (50th percentile): {} msat/byte",
        fee_percentiles,
        fee_per_msat
    );

    Ok(fee_per_msat)
}

// A mock for rubber-stamping signatures.
pub async fn mock_signer_p2tr(
    _key_name: String,
    _derivation_path: Vec<Vec<u8>>,
    _message_hash: Vec<u8>,
) -> Result<Vec<u8>> {
    Ok(vec![255; 64])
}

pub async fn mock_signer_p2pkh(
    _key_name: String,
    _derivation_path: Vec<Vec<u8>>,
    _message_hash: Vec<u8>,
) -> Result<Vec<u8>> {
    Ok(vec![255; 72])
}
