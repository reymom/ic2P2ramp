use bitcoin::{
    consensus::serialize,
    hashes::Hash,
    key::TweakedPublicKey,
    secp256k1::{schnorr::Signature, PublicKey},
    sighash::SighashCache,
    Address, AddressType, ScriptBuf, Sequence, TapSighashType, Transaction, TxOut, Txid, Witness,
};
use ic_cdk::api::management_canister::bitcoin::{MillisatoshiPerByte, Satoshi, Utxo};
use std::str::FromStr;

use crate::api;
use crate::model::types::{
    errors::{BitcoinError, Result},
    wallet::WalletConfig,
};
use crate::wallet::get_fee_per_byte;

use super::transform_network;

/// Returns the P2TR raw key spend address of this canister at the given derivation path.
pub async fn get_address(config: WalletConfig) -> Result<Address> {
    let public_key =
        api::schnorr::schnorr_public_key(config.key_name, config.derivation_path).await?;
    let x_only_pubkey = bitcoin::key::XOnlyPublicKey::from(
        PublicKey::from_slice(&public_key).map_err(BitcoinError::from)?,
    );
    let tweaked_pubkey = TweakedPublicKey::dangerous_assume_tweaked(x_only_pubkey);
    Ok(Address::p2tr_tweaked(
        tweaked_pubkey,
        transform_network(config.network),
    ))
}

/// Sends a P2TR raw key spend transaction to the network that transfers the
/// given amount to the given destination, where the source of the funds is the
/// canister itself at the given derivation path.
pub async fn send_key_spend(
    config: WalletConfig,
    dst_address: String,
    amount: Satoshi,
) -> Result<Txid> {
    let fee_per_byte = get_fee_per_byte(config.network).await?;

    // Fetch P2PKH address and UTXOs.
    let own_address = get_address(config.clone()).await?;
    ic_cdk::println!("[send_key_spend] own_address = {:?}", own_address);

    // Get utxos up to necessary amount HERE?
    let own_utxos = api::bitcoin::get_utxos(config.network, own_address.to_string()).await?;
    ic_cdk::println!("[send_key_spend] utxos = {:?}", own_utxos);

    let dst_address = Address::from_str(&dst_address)
        .map_err(BitcoinError::from)?
        .require_network(transform_network(config.network))
        .map_err(BitcoinError::from)?;

    // Build the transaction that sends `amount` to the destination address.
    let (transaction, prevouts) =
        build_p2tr_key_path_spend_tx(&own_address, &own_utxos, &dst_address, amount, fee_per_byte)
            .await?;

    let tx_bytes = serialize(&transaction);
    ic_cdk::println!(
        "[send_key_spend] Transaction to sign: {}",
        hex::encode(tx_bytes)
    );

    // Sign the transaction.
    let signed_transaction = schnorr_sign_key_spend_transaction(
        &own_address,
        transaction,
        prevouts.as_slice(),
        config.key_name,
        config.derivation_path,
        api::schnorr::sign_with_schnorr,
    )
    .await?;

    let signed_transaction_bytes = serialize(&signed_transaction);
    ic_cdk::println!(
        "[send_key_spend] Signed transaction: {}",
        hex::encode(&signed_transaction_bytes)
    );

    api::bitcoin::send_transaction(config.network, signed_transaction_bytes).await?;
    Ok(signed_transaction.compute_txid())
}

// Builds a transaction to send the given `amount` of satoshis to the
// destination address.
async fn build_p2tr_key_path_spend_tx(
    own_address: &Address,
    own_utxos: &[Utxo],
    dst_address: &Address,
    amount: Satoshi,
    fee_per_vbyte: MillisatoshiPerByte,
) -> Result<(Transaction, Vec<TxOut>)> {
    // We have a chicken-and-egg problem where we need to know the length
    // of the transaction in order to compute its proper fee, but we need
    // to know the proper fee in order to figure out the inputs needed for
    // the transaction.
    //
    // We solve this problem iteratively. We start with a fee of zero, build
    // and sign a transaction, see what its size is, and then update the fee,
    // rebuild the transaction, until the fee is set to the correct amount.
    ic_cdk::println!("Building transaction...");
    let mut total_fee = 0;
    loop {
        let (transaction, prevouts) = super::helpers::build_transaction_with_fee(
            own_utxos,
            own_address,
            dst_address,
            amount,
            total_fee,
            crate::TransactionType::SimpleTaprootBitcoin,
        )?;

        // Sign the transaction. In this case, we only care about the size
        // of the signed transaction, so we use a mock signer here for efficiency.
        let signed_transaction = schnorr_sign_key_spend_transaction(
            own_address,
            transaction.clone(),
            &prevouts,
            String::from(""), // mock key name
            vec![],           // mock derivation path
            super::helpers::mock_signer_p2tr,
        )
        .await?;

        let tx_vsize = signed_transaction.vsize() as u64;

        if (tx_vsize * fee_per_vbyte) / 1000 == total_fee {
            ic_cdk::println!("Transaction built with fee {}.", total_fee);
            return Ok((transaction, prevouts));
        } else {
            total_fee = (tx_vsize * fee_per_vbyte) / 1000;
        }
    }
}

// Sign a P2TR key spend transaction.
//
// IMPORTANT: This method is for demonstration purposes only and it only
// supports signing transactions if:
//
// 1. All the inputs are referencing outpoints that are owned by `own_address`.
// 2. `own_address` is a P2TR script path spend address.
async fn schnorr_sign_key_spend_transaction<SignFun, Fut>(
    own_address: &Address,
    mut transaction: Transaction,
    prevouts: &[TxOut],
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

        let signing_data = sighasher
            .taproot_key_spend_signature_hash(
                i,
                &bitcoin::sighash::Prevouts::All(prevouts),
                TapSighashType::Default,
            )
            .map_err(|e| {
                BitcoinError::InternalError(format!("P2tr raw key spend sighash error: {:?}", e))
            })?
            .as_byte_array()
            .to_vec();

        let raw_signature = signer(
            key_name.clone(),
            derivation_path.clone(),
            signing_data.clone(),
        )
        .await?;

        // Update the witness stack.
        let witness = sighasher.witness_mut(i).unwrap();
        let signature = bitcoin::taproot::Signature {
            signature: Signature::from_slice(&raw_signature).expect("failed to parse signature"),
            sighash_type: TapSighashType::Default,
        };
        witness.push(signature.to_vec());
    }

    Ok(transaction)
}
