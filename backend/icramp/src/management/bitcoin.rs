use base64::{Engine, engine::general_purpose::STANDARD};
use bitcoin_backend::types::{RuneID, RuneUTXOEntry};
use std::{collections::HashMap, str::FromStr, time::Duration};

use bitcoin::{
    self, AddressType,
    address::Address,
    hashes::{Hash, sha256, sha256d},
    secp256k1::Scalar,
};
use secp256k1::{
    Message, PublicKey, Secp256k1,
    ecdsa::{RecoverableSignature, RecoveryId, Signature},
};

use crate::{
    inter_canister::bitcoin::{
        bitcoin_backend_cancel_deposit, bitcoin_backend_complete_order,
        bitcoin_backend_deposit_funds,
    },
    model::{
        errors::{BlockchainError, Result, SystemError, UserError},
        memory::stable::{
            orders::{cancel_order, unset_processing_order},
            spent_transactions::mark_tx_hash_as_processed,
        },
        types::{
            AddressType as CommonAddressType, BlockchainAsset, PaymentProvider,
            PaymentProviderType, TransactionAddress, unisat::UnisatTxOut,
        },
    },
    outcalls::unisat::{
        transaction::{fetch_unisat_tx_outs, fetch_unisat_tx_status},
        utxo::fetch_rune_utxo_balance,
    },
};

const MAX_ATTEMPTS: u32 = 100; // 100 * ~2 min avg = ~3 hours
const BASE_INTERVAL_SECS: u64 = 30; // Initial retry interval (30 sec)
const MAX_INTERVAL_SECS: u64 = 600; // Max retry interval (10 min)

#[derive(Clone)]
pub enum BitcoinTransactionAction {
    DepositFunds {
        offramper_providers: HashMap<PaymentProviderType, PaymentProvider>,
        offramper_address: TransactionAddress,
        offramper_id: u64,
        asset: BlockchainAsset,
        currency: String,
        amount: u128,
    },
    CompleteOrder {
        order_id: u64,
        amount: u64,
        onramper_address: String,
    },
    CancelOrder {
        order_id: u64,
        amount: u64,
        offramper_address: String,
    },
}

impl BitcoinTransactionAction {
    fn validate_rune_amount(&self, runes: Vec<RuneUTXOEntry>) -> bool {
        let total_rune_amount: u64 = runes.iter().map(|r| r.rune_amount).sum();
        ic_cdk::println!(
            "[validate_rune_amount] total_rune_amount = {}",
            total_rune_amount
        );

        match self {
            BitcoinTransactionAction::DepositFunds { amount, .. } => {
                total_rune_amount == *amount as u64
            }
            BitcoinTransactionAction::CompleteOrder { amount, .. } => total_rune_amount == *amount,
            BitcoinTransactionAction::CancelOrder { amount, .. } => total_rune_amount == *amount,
        }
    }
}

pub fn spawn_bitcoin_tx_listener(
    txid: String,
    action: BitcoinTransactionAction,
    dst_address: String,
    rune_id: Option<RuneID>,
    attempt: u32,
) {
    if attempt >= MAX_ATTEMPTS {
        ic_cdk::println!(
            "[spawn_bitcoin_tx_listener] Max attempts reached for tx {}. Stopping.",
            txid
        );
        return;
    }

    let txid_clone = txid.clone();
    let next_attempt = attempt + 1;
    let retry_interval = (BASE_INTERVAL_SECS * 2_u64.pow(attempt.min(10))) // Exponential growth
        .min(MAX_INTERVAL_SECS); // Cap at max interval

    ic_cdk::println!(
        "[spawn_bitcoin_tx_listener] Checking tx: {} with interval: {}",
        txid_clone,
        retry_interval
    );
    ic_cdk_timers::set_timer(Duration::from_secs(retry_interval), move || {
        ic_cdk::spawn(async move {
            ic_cdk::println!(
                "[spawn_bitcoin_tx_listener] Attempt {}: Checking tx: {}",
                attempt,
                txid_clone
            );
            match fetch_unisat_tx_status(&txid_clone).await {
                Ok(true) => {
                    ic_cdk::println!(
                        "[spawn_bitcoin_tx_listener] Transaction {} confirmed",
                        txid_clone
                    );

                    match action.clone() {
                        BitcoinTransactionAction::DepositFunds {
                            offramper_providers,
                            offramper_address,
                            offramper_id,
                            asset,
                            currency,
                            amount,
                        } => {
                            // If this is a rune transaction, fetch UTXOs by filtering on the destination address.
                            let runes = if rune_id.is_some() {
                                match fetch_utxos_for_order(&txid_clone, &dst_address).await {
                                    Ok(r) if !r.is_empty() => {
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] Rune UTXOs = {:?}",
                                            r
                                        );
                                        r
                                    }
                                    _ => {
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] No rune UTXOs found for tx {}",
                                            txid_clone
                                        );
                                        return;
                                    }
                                }
                            } else {
                                Vec::new()
                            };
                            if !action.validate_rune_amount(runes.clone()) {
                                ic_cdk::println!(
                                    "[spawn_bitcoin_tx_listener] Error: Rune amount mismatch for tx {}",
                                    txid_clone
                                );
                                return;
                            }
                            match bitcoin_backend_deposit_funds(
                                offramper_address.clone().address,
                                amount as u64,
                                rune_id.clone(),
                            )
                            .await
                            {
                                Ok(()) => {
                                    match super::order::create_order(
                                        &currency,
                                        offramper_id,
                                        offramper_address,
                                        offramper_providers,
                                        asset,
                                        amount,
                                        None,
                                        None,
                                        Some(runes),
                                    )
                                    .await
                                    {
                                        Ok(order_id) => {
                                            // Mark tx as processed to avoid double processing
                                            mark_tx_hash_as_processed(txid);
                                            ic_cdk::println!(
                                                "[spawn_bitcoin_tx_listener] bitcoin order created, order id = {}",
                                                order_id
                                            )
                                        }
                                        Err(e) => ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] Error creating order: {:?}",
                                            e
                                        ),
                                    }
                                }
                                Err(e) => ic_cdk::println!(
                                    "[spawn_bitcoin_tx_listener] Error depositing funds to bitcoin backend: {:?}",
                                    e
                                ),
                            }
                        }
                        BitcoinTransactionAction::CompleteOrder {
                            onramper_address,
                            amount,
                            order_id,
                        } => {
                            match bitcoin_backend_complete_order(
                                onramper_address,
                                amount as u64,
                                rune_id.clone(),
                            )
                            .await
                            {
                                Ok(()) => match super::order::set_order_completed(order_id) {
                                    Ok(()) => {
                                        let _ = unset_processing_order(&order_id);
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] bitcoin order completed, order id = {}",
                                            order_id
                                        )
                                    }
                                    Err(e) => {
                                        let _ = unset_processing_order(&order_id);
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] Error setting order as completed: {:?}",
                                            e
                                        )
                                    }
                                },
                                Err(e) => {
                                    let _ = unset_processing_order(&order_id);
                                    ic_cdk::println!(
                                        "[spawn_bitcoin_tx_listener] Error completing order: {:?}",
                                        e
                                    )
                                }
                            }
                        }
                        BitcoinTransactionAction::CancelOrder {
                            offramper_address,
                            amount,
                            order_id,
                        } => {
                            match bitcoin_backend_cancel_deposit(
                                offramper_address,
                                amount as u64,
                                rune_id.clone(),
                            )
                            .await
                            {
                                Ok(()) => match cancel_order(order_id) {
                                    Ok(()) => {
                                        let _ = unset_processing_order(&order_id);
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] bitcoin order cancelled, order id = {}",
                                            order_id
                                        )
                                    }
                                    Err(e) => {
                                        let _ = unset_processing_order(&order_id);
                                        ic_cdk::println!(
                                            "[spawn_bitcoin_tx_listener] Error cancelling order: {:?}",
                                            e
                                        )
                                    }
                                },
                                Err(e) => {
                                    let _ = unset_processing_order(&order_id);
                                    ic_cdk::println!(
                                        "[spawn_bitcoin_tx_listener] Error cancelling deposit: {:?}",
                                        e
                                    )
                                }
                            }
                        }
                    }
                }
                Ok(false) => {
                    ic_cdk::println!(
                        "[spawn_bitcoin_tx_listener] Transaction {} not yet confirmed",
                        txid_clone
                    );
                    // Reschedule the check if not confirmed.
                    spawn_bitcoin_tx_listener(
                        txid_clone,
                        action,
                        dst_address,
                        rune_id,
                        next_attempt,
                    );
                }
                Err(e) => {
                    ic_cdk::println!(
                        "[spawn_bitcoin_tx_listener] Error checking tx status: {:?}",
                        e
                    );
                    spawn_bitcoin_tx_listener(
                        txid_clone,
                        action,
                        dst_address,
                        rune_id,
                        next_attempt,
                    );
                }
            }
        });
    });
}

pub async fn fetch_utxos_for_order(txid: &str, dst_address: &str) -> Result<Vec<RuneUTXOEntry>> {
    let tx_outs = fetch_unisat_tx_outs(txid).await?;
    // Filter outputs by the given destination address
    let filtered_outs: Vec<UnisatTxOut> = tx_outs
        .into_iter()
        .filter(|out| out.address == dst_address)
        .collect();
    ic_cdk::println!(
        "[fetch_utxos_for_order] filtered_outs for address {}: {:?}",
        dst_address,
        filtered_outs
    );
    let mut rune_utxos: Vec<RuneUTXOEntry> = Vec::new();
    for utxo in filtered_outs {
        match fetch_rune_utxo_balance(&utxo.txid, utxo.vout).await {
            Ok(rune_balance) => {
                if rune_balance > 0 {
                    rune_utxos.push(RuneUTXOEntry {
                        txid: utxo.txid.clone(),
                        vout: utxo.vout,
                        rune_amount: rune_balance as u64,
                        script_pubkey: utxo.script_pk.clone(),
                    });
                }
            }
            Err(e) => {
                println!(
                    "Error when fetching UTXO {}:{}, {:?}",
                    utxo.txid, utxo.vout, e
                );
            }
        }
    }
    Ok(rune_utxos)
}

/// Verifies a Bitcoin signature for a given message and public key.
///
/// This function performs the following steps:
/// 1. Validates that the provided Bitcoin address corresponds to the given public key.
/// 2. Decodes the signature from Base64 format and extracts the compact signature and recovery ID.
/// 3. Computes the Bitcoin-specific magic hash of the message.
/// 4. Recovers the public key from the signature and verifies it matches the provided public key.
/// 5. (Optional) Verifies the signature against the message and public key as a redundant check.
///
/// # Arguments
///
/// - `btc_address`: The Bitcoin address associated with the public key.
/// - `message`: The message that was signed.
/// - `signature`: The Base64-encoded compact signature of the message.
/// - `pubkey`: The public key as a string.
///
/// # Returns
///
/// - `Ok(())` if the signature is valid and corresponds to the provided Bitcoin address and public key.
/// - `Err(UserError::InvalidSignature)` if the signature validation fails.
/// - `Err(SystemError)` if there are issues with decoding or parsing.
///
/// # Errors
///
/// - Returns `SystemError::ParseError` if the signature or public key is malformed.
/// - Returns `SystemError::InvalidInput` if the signature length is incorrect.
/// - Returns `BlockchainError::InvalidPubKey` if the provided public key is invalid.
/// - Returns `UserError::InvalidSignature` if the signature does not match the public key or address.
///
/// # Example
///
/// ```rust
/// let btc_address = "tb1qexampleaddress...";
/// let message = "Example message to be signed";
/// let signature = "Base64EncodedCompactSignature";
/// let pubkey = "PublicKeyInHex";
///
/// match verify_signature(btc_address, message, signature, pubkey) {
///     Ok(()) => println!("Signature is valid!"),
///     Err(e) => eprintln!("Signature validation failed: {:?}", e),
/// }
/// ```
pub fn verify_signature(
    btc_address: &str,
    message: &str,
    signature: &str,
    pubkey: &str,
) -> Result<()> {
    // Check if the public key corresponds to the address
    validate_address_with_pubkey(btc_address, pubkey)?;

    // Verify message signature
    let signature_bytes = STANDARD.decode(signature).map_err(|_| {
        SystemError::ParseError("Cannot decode Base64 Bitcoin Signature".to_string())
    })?;
    if signature_bytes.len() != 65 {
        return Err(SystemError::InvalidInput("Invalid signature length".to_string()).into());
    }

    let (recovery_id_byte, compact_sig) = signature_bytes.split_at(1);
    let recovery_id = RecoveryId::try_from((recovery_id_byte[0] as i32 - 27) % 4)
        .map_err(|_| SystemError::ParseError("Invalid recovery id".to_string()))?;
    ic_cdk::println!("Recovery ID: {:?}", recovery_id);

    let recoverable_sig = RecoverableSignature::from_compact(compact_sig, recovery_id)
        .map_err(|_| SystemError::ParseError("Invalid compact signature".to_string()))?;

    // Use magic hash for message hashing
    let magic_hashed_message = compute_magic_hash(message);
    let msg = Message::from_digest_slice(magic_hashed_message.as_ref()).map_err(|e| {
        SystemError::ParseError(format!(
            "Failed to parse signature's message hash: {}",
            e.to_string()
        ))
    })?;

    let provided_pubkey =
        PublicKey::from_str(pubkey).map_err(|_| BlockchainError::InvalidPubKey)?;
    let recovered_pubkey = recoverable_sig.recover(&msg).map_err(|e| {
        ic_cdk::println!(
            "Failed to recover public key from signature: {}",
            e.to_string()
        );
        return UserError::InvalidSignature;
    })?;
    if recovered_pubkey != provided_pubkey {
        return Err(UserError::InvalidSignature.into());
    }

    // Verify the signature (redundant check, recovering the pubkey is enough)
    let sig = Signature::from_compact(&recoverable_sig.serialize_compact().1)
        .map_err(|_| SystemError::ParseError("Cannot parse Bitcoin Signature".to_string()))?;
    let secp = Secp256k1::verification_only();
    secp.verify_ecdsa(&msg, &sig, &provided_pubkey)
        .map_err(|_| UserError::InvalidSignature)?;

    Ok(())
}

/// Validates if a Bitcoin address corresponds to a given public key.
/// Supports both regular and Taproot (XOnly) public keys.
///
/// # Arguments
/// - `btc_address`: The Bitcoin address to validate.
/// - `pubkey`: The Bitcoin public key.
///
/// # Returns
/// - `Ok(())` if the address is valid for either the regular or XOnly public key.
/// - `Err(UserError::InvalidSignature)` if the address does not match the provided public keys.
fn validate_address_with_pubkey(btc_address: &str, pubkey: &str) -> Result<()> {
    let address = Address::from_str(btc_address)
        .map_err(|_| BlockchainError::InvalidAddress(CommonAddressType::Bitcoin))?
        .assume_checked();

    ic_cdk::println!(
        "Address: {:?} of type {:?}",
        address,
        address.address_type()
    );
    match address.address_type() {
        Some(AddressType::P2tr) => {
            // Decode the provided public key and slice to XOnly
            let pubkey_bytes = hex::decode(pubkey).map_err(|_| BlockchainError::InvalidPubKey)?;
            let xonly_bytes = &pubkey_bytes[1..33];
            let xonly_pubkey = bitcoin::XOnlyPublicKey::from_slice(xonly_bytes)
                .map_err(|_| BlockchainError::InvalidPubKey)?;

            // Compute the tweaked XOnly public key
            let tweaked_pubkey = compute_tweaked_pubkey(&xonly_pubkey, None)?;

            // Extract the address payload
            let script_pubkey = address.script_pubkey();
            let address_payload = &script_pubkey.as_bytes()[2..34]; // Skip OP_1 and OP_PUSH32

            // Compare the XOnly public key to the address payload
            if tweaked_pubkey.serialize().to_vec() == address_payload {
                ic_cdk::println!("Address matches the tweaked XOnly public key.");
            } else {
                ic_cdk::println!("Address does not match the XOnly public key.");
                return Err(BlockchainError::InvalidAddress(CommonAddressType::Bitcoin).into());
            }

            // Additional check for safety
            if address.is_related_to_xonly_pubkey(&tweaked_pubkey) {
                Ok(())
            } else {
                Err(BlockchainError::InvalidAddress(CommonAddressType::Bitcoin).into())
            }
        }
        _ => {
            let pubkey =
                bitcoin::PublicKey::from_str(pubkey).map_err(|_| BlockchainError::InvalidPubKey)?;
            ic_cdk::println!(
                "is_related_to_pubkey: {:?}",
                address.is_related_to_pubkey(&pubkey)
            );
            if address.is_related_to_pubkey(&pubkey) {
                Ok(())
            } else {
                Err(BlockchainError::InvalidAddress(CommonAddressType::Bitcoin).into())
            }
        }
    }
}

/// Computes the tweaked XOnly public key for Taproot.
///
/// # Arguments
/// - `xonly_pubkey`: The XOnly public key to tweak.
/// - `script_hash`: Optional script commitment or Merkle root for the tweak.
///
/// # Returns
/// - `Ok(tweaked_pubkey)` if the tweak is successful.
/// - `Err(BlockchainError)` otherwise.
fn compute_tweaked_pubkey(
    xonly_pubkey: &bitcoin::XOnlyPublicKey,
    script_hash: Option<&[u8]>,
) -> Result<bitcoin::XOnlyPublicKey> {
    // Compute the tweak hash
    let tag = b"TapTweak";
    let tag_hash = sha256::Hash::hash(tag);

    let tweak_input = if let Some(script_hash) = script_hash {
        [xonly_pubkey.serialize().as_slice(), script_hash].concat()
    } else {
        xonly_pubkey.serialize().to_vec()
    };

    let mut hash_input = Vec::new();
    hash_input.extend_from_slice(tag_hash.as_ref());
    hash_input.extend_from_slice(tag_hash.as_ref());
    hash_input.extend_from_slice(&tweak_input);

    let tweak = sha256::Hash::hash(&hash_input);

    // Convert tweak to Scalar
    let tweak_scalar =
        Scalar::from_be_bytes(tweak.to_byte_array()).map_err(|_| BlockchainError::InvalidPubKey)?;

    // Apply the tweak
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let (tweaked_pubkey, _) = xonly_pubkey
        .add_tweak(&secp, &tweak_scalar)
        .map_err(|_| BlockchainError::InvalidPubKey)?;

    Ok(tweaked_pubkey)
}

fn compute_magic_hash(message: &str) -> sha256d::Hash {
    // Bitcoin Signed Message prefix
    let prefix = b"Bitcoin Signed Message:\n";
    let prefix_length = encode_varint(prefix.len() as u64);
    let message_length = encode_varint(message.len() as u64);

    // Build the combined buffer
    let mut magic_message = Vec::new();
    magic_message.extend_from_slice(&prefix_length);
    magic_message.extend_from_slice(prefix);
    magic_message.extend_from_slice(&message_length);
    magic_message.extend_from_slice(message.as_bytes());

    // Double SHA256
    sha256d::Hash::hash(&magic_message)
}

// Helper function to encode a varint
fn encode_varint(value: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    if value < 0xFD {
        buf.push(value as u8);
    } else if value <= 0xFFFF {
        buf.push(0xFD);
        buf.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= 0xFFFFFFFF {
        buf.push(0xFE);
        buf.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        buf.push(0xFF);
        buf.extend_from_slice(&(value as u64).to_le_bytes());
    }
    buf
}
