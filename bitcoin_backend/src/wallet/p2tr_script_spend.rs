use bitcoin::{
    consensus::serialize,
    hashes::Hash,
    key::Secp256k1,
    script::PushBytesBuf,
    secp256k1::{schnorr::Signature, PublicKey},
    sighash::SighashCache,
    taproot::{ControlBlock, LeafVersion, TaprootBuilder},
    Address, AddressType, ScriptBuf, Sequence, TapLeafHash, TapSighashType, Transaction, TxOut,
    Txid, Witness, XOnlyPublicKey,
};
use ic_cdk::api::management_canister::bitcoin::{MillisatoshiPerByte, Satoshi, Utxo};
use std::str::FromStr;

use crate::{
    api::schnorr::schnorr_public_key,
    model::types::{
        errors::{BitcoinError, Result},
        wallet::{TaprootUseCase, WalletConfig},
    },
    TransactionType,
};

fn build_script_for_use_case(
    x_only_pubkey: &XOnlyPublicKey,
    use_case: TaprootUseCase,
) -> Result<ScriptBuf> {
    match use_case {
        TaprootUseCase::Standard => Ok(bitcoin::blockdata::script::Builder::new()
            .push_x_only_key(x_only_pubkey)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .into_script()),
        TaprootUseCase::RuneTransfer(symbol) => {
            let mut symbol_bytes = PushBytesBuf::new();
            symbol_bytes
                .extend_from_slice(symbol.as_bytes())
                .map_err(|_| BitcoinError::InternalError("Invalid Rune symbol".to_string()))?;
            Ok(bitcoin::blockdata::script::Builder::new()
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
                .push_slice(symbol_bytes.as_push_bytes())
                .into_script())
        }
        TaprootUseCase::Inscription(data) => {
            let mut inscription_bytes = PushBytesBuf::new();
            inscription_bytes
                .extend_from_slice(&data)
                .map_err(|_| BitcoinError::InternalError("Invalid inscription data".to_string()))?;
            Ok(bitcoin::blockdata::script::Builder::new()
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
                .push_slice(inscription_bytes.as_push_bytes())
                .into_script())
        }
    }
}

/// Returns the P2TR script spend address for all runes.
pub async fn get_address(config: WalletConfig) -> Result<Address> {
    let public_key = schnorr_public_key(config.key_name, config.derivation_path).await?;

    let x_only_pubkey = bitcoin::key::XOnlyPublicKey::from(
        PublicKey::from_slice(&public_key).map_err(BitcoinError::from)?,
    );

    let secp_engine = Secp256k1::new();
    let taproot_spend_info = TaprootBuilder::new()
        .finalize(&secp_engine, x_only_pubkey)
        .map_err(BitcoinError::from)?;

    let address = Address::p2tr_tweaked(
        taproot_spend_info.output_key(),
        crate::wallet::transform_network(config.network),
    );

    Ok(address)
}

/// Returns the P2TR script spend address for this canister at the given derivation path.
// pub async fn get_address_with_info(
//     config: WalletConfig,
//     use_case: TaprootUseCase,
// ) -> Result<(Address, TaprootSpendInfo, ScriptBuf)> {
//     let public_key = schnorr_public_key(config.key_name, config.derivation_path).await?;

//     let x_only_pubkey = bitcoin::key::XOnlyPublicKey::from(
//         PublicKey::from_slice(&public_key).map_err(BitcoinError::from)?,
//     );

//     let script = build_script_for_use_case(&x_only_pubkey, use_case)?;
//     let secp_engine = Secp256k1::new();
//     let taproot_spend_info = TaprootBuilder::new()
//         .add_leaf(0, script.clone())
//         .map_err(BitcoinError::from)?
//         .finalize(&secp_engine, x_only_pubkey)
//         .map_err(BitcoinError::from)?;

//     let address = Address::p2tr_tweaked(
//         taproot_spend_info.output_key(),
//         crate::wallet::transform_network(config.network),
//     );
//     Ok((address, taproot_spend_info, script))
// }

/// Sends BTC or Runes using a Taproot script-spend address.
pub async fn send_script_spend(
    config: WalletConfig,
    taproot_use_case: TaprootUseCase,
    dst_address: String,
    amount: Satoshi,
    tx_type: TransactionType,
) -> Result<Txid> {
    let fee_per_byte = crate::wallet::get_fee_per_byte(config.network).await?;

    let address = get_address(config.clone()).await?;
    ic_cdk::println!("[send_script_spend] address = {:?}", address);

    // Step 1: Generate Taproot address.
    // let (address, taproot_spend_info, script) =
    //     get_address_with_info(config.clone(), taproot_use_case).await?;
    // ic_cdk::println!("[send_script_spend] address = {:?}", address);

    // ✅ Step 1: Get the script that matches the transaction type
    let public_key =
        schnorr_public_key(config.key_name.clone(), config.derivation_path.clone()).await?;
    let x_only_pubkey = bitcoin::key::XOnlyPublicKey::from(
        PublicKey::from_slice(&public_key).map_err(BitcoinError::from)?,
    );
    let script = build_script_for_use_case(&x_only_pubkey, taproot_use_case)?;

    // ✅ Step 3: Generate spend info based on the selected script
    let secp_engine = Secp256k1::new();
    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, script.clone())
        .map_err(BitcoinError::from)?
        .finalize(&secp_engine, x_only_pubkey)
        .map_err(BitcoinError::from)?;

    let script_map = taproot_spend_info.script_map();
    let (leaf_script, leaf_version) = script_map
        .keys()
        .next()
        .ok_or_else(|| BitcoinError::InternalError("No script found in script map".to_string()))?;
    let control_block = taproot_spend_info
        .control_block(&(leaf_script.clone(), *leaf_version))
        .ok_or_else(|| BitcoinError::InternalError("Missing ControlBlock".to_string()))?;

    // Step 4: Fetch UTXOs for the Taproot address.
    let own_utxos = crate::api::bitcoin::get_utxos(config.network, address.to_string()).await?;
    ic_cdk::println!("[send_script_spend] utxos = {:?}", own_utxos);

    // ✅ Step 5: Build & Sign the Transaction
    let dst_address = Address::from_str(&dst_address)
        .map_err(BitcoinError::from)?
        .require_network(crate::wallet::transform_network(config.network))
        .map_err(|e| BitcoinError::UnsupportedAddressType(e.to_string()))?;

    let (transaction, prevouts) = build_p2tr_transaction(
        &address,
        &control_block,
        &leaf_script,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
        tx_type,
    )
    .await?;

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

    crate::api::bitcoin::send_transaction(config.network, signed_transaction_bytes).await?;

    Ok(signed_transaction.compute_txid())
}

/// Builds a P2TR transaction to send the given amount to the destination.
pub async fn build_p2tr_transaction(
    own_address: &Address,
    control_block: &ControlBlock,
    script: &ScriptBuf,
    utxos: &[Utxo],
    dst_address: &Address,
    amount: Satoshi,
    fee_per_byte: MillisatoshiPerByte,
    tx_type: TransactionType,
) -> Result<(Transaction, Vec<TxOut>)> {
    ic_cdk::println!("Building transaction...");
    let mut total_fee = 0;
    loop {
        let (transaction, prevouts) = super::helpers::build_transaction_with_fee(
            utxos,
            own_address,
            dst_address,
            amount,
            total_fee,
            tx_type.clone(),
        )?;

        let signed_transaction = schnorr_sign_transaction(
            transaction.clone(),
            prevouts.as_slice(),
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
#[allow(clippy::too_many_arguments)]
pub async fn schnorr_sign_transaction<SignFun, Fut>(
    mut transaction: Transaction,
    prevouts: &[TxOut],
    own_address: &Address,
    control_block: &ControlBlock,
    script: &ScriptBuf,
    key_name: String,
    derivation_path: Vec<Vec<u8>>,
    signer: SignFun,
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
            )
            .map_err(BitcoinError::from)?;

        let raw_signature = signer(
            key_name.clone(),
            derivation_path.clone(),
            sighash.as_byte_array().to_vec(),
        )
        .await?;

        let signature = bitcoin::taproot::Signature {
            signature: Signature::from_slice(&raw_signature).map_err(BitcoinError::from)?,
            sighash_type: TapSighashType::Default,
        };

        let witness = sighasher.witness_mut(i).unwrap();
        witness.push(signature.to_vec());
        witness.push(script.to_bytes());
        witness.push(control_block.serialize());
    }

    Ok(transaction)
}
