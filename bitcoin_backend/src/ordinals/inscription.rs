use serde::Serialize;

use bitcoin::Txid;
use ic_cdk::api::management_canister::bitcoin::Satoshi;

use crate::model::types::{
    errors::Result,
    wallet::{TaprootUseCase, WalletConfig},
};

#[derive(Serialize)]
pub struct Inscription {
    pub content: String,
    pub content_type: String,
    pub metadata: Option<String>,
}

/// Sends an Ordinals inscription to the Bitcoin network.
///
/// # Arguments
/// - `network`: The Bitcoin network (e.g., Testnet, Mainnet).
/// - `key_name`: The name of the Schnorr key.
/// - `derivation_path`: The derivation path for the Taproot address.
/// - `inscription`: The inscription content (e.g., Ordinals data).
/// - `dst_address`: The destination address.
/// - `amount`: The amount of BTC to send.
///
/// # Returns
/// - `Result<Txid>`: The transaction ID of the inscription transaction.
// pub async fn send_inscription(
//     network: BitcoinNetwork,
//     key_name: String,
//     derivation_path: Vec<Vec<u8>>,
//     inscription: Vec<u8>,
//     dst_address: String,
//     amount: Satoshi,
// ) -> Result<Txid> {
//     // Step 1: Generate Taproot address.
//     let public_key = schnorr_public_key(key_name.clone(), derivation_path.clone()).await?;
//     let (address, taproot_spend_info) =
//         generate_taproot_address(network, &public_key, &inscription)?;

//     // Step 2: Fetch UTXOs for the Taproot address.
//     let own_utxos = crate::api::bitcoin::get_utxos(network, address.to_string()).await?;

//     // Step 3: Construct the transaction.
//     let script_map = taproot_spend_info.script_map();
//     let (script, leaf_version) = script_map
//         .keys()
//         .next()
//         .ok_or_else(|| BitcoinError::InternalError("No script found in script map".to_string()))?;

//     let control_block = taproot_spend_info
//         .control_block(&(script.clone(), *leaf_version))
//         .ok_or_else(|| BitcoinError::InternalError("Missing ControlBlock".to_string()))?;

//     let dst_address = Address::from_str(&dst_address)
//         .map_err(BitcoinError::from)?
//         .require_network(crate::wallet::transform_network(network))
//         .map_err(|e| BitcoinError::UnsupportedAddressType(e.to_string()))?;
//     let fee_per_byte = crate::wallet::get_fee_per_byte(network).await?;
//     let (transaction, prevouts) = build_p2tr_transaction(
//         &address,
//         &control_block,
//         script,
//         &own_utxos,
//         &dst_address,
//         amount,
//         fee_per_byte,
//     )
//     .await?;

//     // Step 4: Sign the transaction.
//     let signed_transaction = schnorr_sign_transaction(
//         transaction,
//         &prevouts,
//         &address,
//         &control_block,
//         script,
//         key_name,
//         derivation_path,
//         crate::api::schnorr::sign_with_schnorr,
//     )
//     .await?;

//     // Step 5: Broadcast the transaction.
//     let signed_transaction_bytes = serialize(&signed_transaction);
//     crate::api::bitcoin::send_transaction(network, signed_transaction_bytes).await?;

//     Ok(signed_transaction.compute_txid())
// }

pub async fn send_inscription(
    config: WalletConfig,
    inscription: Vec<u8>,
    dst_address: String,
    amount: Satoshi,
) -> Result<Txid> {
    crate::wallet::p2tr_script_spend::send_script_spend(
        config,
        TaprootUseCase::Inscription(inscription),
        dst_address,
        amount,
        //should define new type for inscriptions
        crate::TransactionType::ScriptedTaprootBitcoin,
    )
    .await
}
