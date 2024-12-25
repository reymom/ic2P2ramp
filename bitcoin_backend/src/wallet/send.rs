use bitcoin::{script::PushBytesBuf, Txid};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Satoshi};

use crate::{memory::heap::config, model::types::{errors::{BitcoinError, Result}, runes::RuneID}};

pub async fn send_btc_or_rune(
    network: BitcoinNetwork,
    derivation_path: Vec<Vec<u8>>,
    key_name: String,
    dst_address: String,
    amount: Satoshi,
    rune: Option<RuneID>,
    use_taproot: bool,
) -> Result<Txid> {
    if let Some(rune) = rune {
        let rune_symbol = config::get_rune_metadata(&rune)?.symbol;

        let mut symbol_bytes = PushBytesBuf::new();
        symbol_bytes
            .extend_from_slice(rune_symbol.as_bytes())
            .map_err(|_| BitcoinError::InternalError("Invalid Rune symbol".to_string()))?;

        let rune_script = bitcoin::blockdata::script::Builder::new()
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
            .push_slice(symbol_bytes.as_push_bytes())
            .into_script();

        crate::wallet::p2tr_script_spend::send_script_spend(
            network,
            key_name,
            derivation_path,
            rune_script,
            dst_address,
            amount,
        )
        .await
    } else if use_taproot {
        crate::wallet::p2tr_raw_key_spend::send_key_spend(
            network,
            derivation_path,
            key_name,
            dst_address,
            amount,
        )
        .await
    } else {
        crate::wallet::p2pkh::send(network, derivation_path, key_name, dst_address, amount).await
    }
}
