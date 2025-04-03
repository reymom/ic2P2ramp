use bitcoin::{
    consensus::serialize,
    hashes::Hash,
    key::Secp256k1,
    opcodes,
    script::{Builder, PushBytesBuf},
    secp256k1::{schnorr::Signature, PublicKey},
    sighash::SighashCache,
    taproot::{ControlBlock, LeafVersion, TaprootBuilder, TaprootSpendInfo},
    Address, AddressType, ScriptBuf, Sequence, TapLeafHash, TapSighashType, Transaction, TxOut,
    Txid, Witness, XOnlyPublicKey,
};
use ic_btc_interface::{MillisatoshiPerByte, Satoshi, Utxo};
use std::str::FromStr;

use crate::{
    api::schnorr::schnorr_public_key,
    model::types::{
        errors::{BitcoinError, Result},
        transfer::TaprootUseCase,
        wallet::WalletConfig,
    },
    ordinals::{inscription::build_ordinal_inscription, runes::build_runestone_etching},
    wallet::utxos::get_tx_utxos,
    TransactionType,
};

pub fn build_script_for_use_case(
    x_only_pubkey: &XOnlyPublicKey,
    use_case: TaprootUseCase,
) -> Result<ScriptBuf> {
    match use_case {
        TaprootUseCase::Standard => Ok(bitcoin::blockdata::script::Builder::new()
            .push_x_only_key(x_only_pubkey)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .into_script()),
        TaprootUseCase::RuneEtching(etching) => {
            let runestone_script = build_runestone_etching(etching)?;

            let runestone_bytes =
                PushBytesBuf::try_from(runestone_script.to_bytes()).map_err(|_| {
                    BitcoinError::InternalError("Failed to encode Runestone script".to_string())
                })?;

            Ok(Builder::new()
                .push_x_only_key(x_only_pubkey)
                .push_opcode(opcodes::all::OP_CHECKSIG)
                .push_opcode(opcodes::OP_FALSE)
                .push_opcode(opcodes::all::OP_IF)
                .push_slice(&runestone_bytes)
                .push_opcode(opcodes::all::OP_ENDIF)
                .into_script())
        }
        TaprootUseCase::Inscription(inscription) => {
            let inscription_script = build_ordinal_inscription(&inscription)?;

            let inscription_bytes =
                PushBytesBuf::try_from(inscription_script.to_bytes()).map_err(|_| {
                    BitcoinError::InternalError("Failed to encode inscription script".to_string())
                })?;

            Ok(bitcoin::blockdata::script::Builder::new()
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
                .push_slice(inscription_bytes.as_push_bytes())
                .into_script())
        }
    }
}

/// Returns the P2TR script spend address for all runes.
pub async fn get_address(
    config: WalletConfig,
    tx_type: TransactionType,
) -> Result<(Address, TaprootSpendInfo)> {
    let public_key = schnorr_public_key(config.key_name, config.derivation_path).await?;
    let x_only_pubkey = bitcoin::key::XOnlyPublicKey::from(
        PublicKey::from_slice(&public_key).map_err(BitcoinError::from)?,
    );

    let use_case = tx_type.to_taproot_use_case();
    let script = if let Some(use_case) = use_case {
        build_script_for_use_case(&x_only_pubkey, use_case)?
    } else {
        return Err(BitcoinError::UnsupportedTransaction);
    };

    let secp_engine = Secp256k1::new();
    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, script.clone())
        .map_err(BitcoinError::from)?
        .finalize(&secp_engine, x_only_pubkey)
        .map_err(BitcoinError::from)?;

    let address = Address::p2tr_tweaked(
        taproot_spend_info.output_key(),
        crate::wallet::transform_network(config.network),
    );

    Ok((address, taproot_spend_info))
}

/// Sends Runes or Ordinals using a Taproot script-spend address.
///
/// 🔹 Currently **only handles the commit transaction**.
///
/// 🔹 Does **NOT** yet handle the reveal transaction.
pub async fn send_script_spend(
    config: WalletConfig,
    dst_address: String,
    amount: Satoshi,
    tx_type: TransactionType,
) -> Result<Txid> {
    let (address, taproot_spend_info) = get_address(config.clone(), tx_type.clone()).await?;
    ic_cdk::println!("[send_script_spend] address = {:?}", address);

    // Generate spend info based on the selected script
    let script_map = taproot_spend_info.script_map();
    let (leaf_script, leaf_version) = script_map
        .keys()
        .next()
        .ok_or_else(|| BitcoinError::InternalError("No script found in script map".to_string()))?;
    let control_block = taproot_spend_info
        .control_block(&(leaf_script.clone(), *leaf_version))
        .ok_or_else(|| BitcoinError::InternalError("Missing ControlBlock".to_string()))?;

    // Fetch UTXOs for the Taproot address.
    let (btc_utxos, rune_utxos) = get_tx_utxos(config.clone(), address.to_string(), None).await?;

    ic_cdk::println!("[send_script_spend] Rune UTXOs = {:?}", rune_utxos);
    ic_cdk::println!("[send_script_spend] BTC UTXOs = {:?}", btc_utxos);

    // Build & Sign the Transaction
    let dst_address = Address::from_str(&dst_address)
        .map_err(BitcoinError::from)?
        .require_network(crate::wallet::transform_network(config.network))
        .map_err(|e| BitcoinError::UnsupportedAddressType(e.to_string()))?;

    let fee_per_byte =
        crate::wallet::get_fee_per_byte(config.network, config.btc_principal).await?;
    let (transaction, prevouts) = build_p2tr_script_spend_transaction(
        tx_type,
        &address,
        &dst_address,
        amount,
        fee_per_byte,
        &btc_utxos,
        &control_block,
        &leaf_script,
    )
    .await?;
    ic_cdk::println!(
        "[send_script_spend] Unsigned Transaction: {}",
        hex::encode(serialize(&transaction))
    );

    let signed_transaction = schnorr_sign_transaction(
        transaction,
        &prevouts,
        &address,
        &control_block,
        &leaf_script,
        config.key_name,
        config.derivation_path,
        crate::api::schnorr::sign_with_schnorr,
    )
    .await?;

    let signed_transaction_bytes = serialize(&signed_transaction);
    ic_cdk::println!(
        "Broadcasting transaction: {}",
        hex::encode(&signed_transaction_bytes)
    );

    crate::api::bitcoin::send_transaction(
        config.network,
        config.btc_principal,
        signed_transaction_bytes,
    )
    .await?;

    // ❌ TODO: Build & send the Reveal transaction
    // ❌ TODO: The commit UTXO must be spent using the script path!
    // ❌ TODO: Verify the Ordinal Inscription or Rune Etching was finalized.

    Ok(signed_transaction.compute_txid())
}

/// Builds a P2TR transaction to send the given amount to the destination.
pub async fn build_p2tr_script_spend_transaction(
    tx_type: TransactionType,
    own_address: &Address,
    dst_address: &Address,
    amount: Satoshi,
    fee_per_byte: MillisatoshiPerByte,
    utxos: &[Utxo],
    control_block: &ControlBlock,
    script: &ScriptBuf,
) -> Result<(Transaction, Vec<TxOut>)> {
    ic_cdk::println!("Building transaction...");
    let mut total_fee = 0;
    loop {
        let (transaction, prevouts) = super::helpers::build_transaction_with_fee(
            tx_type.clone(),
            own_address,
            dst_address,
            amount,
            total_fee,
            utxos,
            None,
        )?;

        let signed_transaction = schnorr_sign_transaction(
            transaction.clone(),
            &prevouts,
            own_address,
            control_block,
            script,
            String::from(""),
            vec![], // mock derivation path
            super::helpers::mock_signer_p2tr,
        )
        .await?;

        let tx_vsize = signed_transaction.vsize() as u64;
        let estimated_fee = (tx_vsize * fee_per_byte) / 1000;

        if estimated_fee == total_fee {
            ic_cdk::println!("Transaction built with fee {}.", total_fee);
            return Ok((transaction, prevouts));
        } else {
            total_fee = estimated_fee;
        }
    }
}

// Sign a P2TR script spend transaction.
//
// IMPORTANT: This method is for demonstration purposes only and it only
// supports signing transactions if:
//
// 1. All the inputs are referencing outpoints that are owned by `own_address`.
// 2. `own_address` is a P2TR script path spend address.
// #[allow(clippy::too_many_arguments)]
pub async fn schnorr_sign_transaction<SignFun, Fut>(
    mut transaction: Transaction,
    prevouts: &[TxOut],
    own_address: &Address,
    control_block: &ControlBlock,
    script: &ScriptBuf,
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    signer: SignFun,
    // x_only_pubkey: XOnlyPublicKey,
) -> Result<Transaction>
where
    SignFun: Fn(String, Vec<Vec<u8>>, Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = Result<Vec<u8>>>,
{
    assert_eq!(own_address.address_type(), Some(AddressType::P2tr),);

    for input in transaction.input.iter_mut() {
        input.script_sig = ScriptBuf::default();
        input.witness = Witness::default();
        input.sequence = Sequence::ENABLE_RBF_NO_LOCKTIME;
    }

    let num_inputs = transaction.input.len();

    for i in 0..num_inputs {
        let mut sighasher = SighashCache::new(&mut transaction);

        let leaf_hash = TapLeafHash::from_script(script, LeafVersion::TapScript);
        let sighash = sighasher
            .taproot_script_spend_signature_hash(
                i,
                &bitcoin::sighash::Prevouts::All(prevouts),
                leaf_hash,
                TapSighashType::Default,
            )?
            .as_byte_array()
            .to_vec();

        let raw_signature = signer(key_name.clone(), derivation_path.clone(), sighash).await?;
        ic_cdk::println!("[DEBUG] Signature for input {}: {:?}", i, raw_signature);

        let signature = bitcoin::taproot::Signature {
            signature: Signature::from_slice(&raw_signature).map_err(BitcoinError::from)?,
            sighash_type: TapSighashType::Default,
        };

        let witness = sighasher.witness_mut(i).unwrap();
        witness.clear();

        witness.push(signature.to_vec());
        witness.push(script.to_bytes());
        witness.push(control_block.serialize());
    }

    Ok(transaction)
}
