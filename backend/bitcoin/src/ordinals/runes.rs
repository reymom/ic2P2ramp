use bitcoin::{
    ScriptBuf, opcodes,
    script::{Builder, PushBytesBuf},
};
use icramp_types::bitcoin::errors::{BitcoinError, Result};

use crate::{RuneID, model::types::runes::Etching};

pub fn build_runestone_edict(
    rune_id: &RuneID,
    amount: u64,
    output_index: u32,
) -> Result<ScriptBuf> {
    let mut payload = Vec::new();
    let (block, tx_index) = rune_id.parts();

    // "Body" Tag (Start of edicts section)
    encode_varint(0, &mut payload);

    // **Encode the Edict (block, tx, amount, output)
    encode_varint(block.into(), &mut payload);
    encode_varint(tx_index.into(), &mut payload);
    encode_varint(amount.into(), &mut payload);
    encode_varint(output_index.into(), &mut payload);

    let payload_pushbytes = PushBytesBuf::try_from(payload).map_err(|_| {
        BitcoinError::InternalError("Failed to encode Runestone payload".to_string())
    })?;

    // OP_RETURN Runestone
    Ok(Builder::new()
        .push_opcode(opcodes::all::OP_RETURN)
        .push_opcode(opcodes::all::OP_PUSHNUM_13) // Runestone Identifier
        .push_slice(&payload_pushbytes) // Encoded payload
        .into_script())
}

pub fn build_runestone_etching(etching: Etching) -> Result<ScriptBuf> {
    let mut payload = Vec::new();

    // Etching Section
    encode_varint(1, &mut payload); // "Etching" Flag (1)

    // Encode Rune Properties
    encode_varint(2, &mut payload); // "Divisibility"
    encode_varint(etching.metadata.divisibility as u128, &mut payload);

    encode_varint(3, &mut payload); // "Premine"
    encode_varint(etching.metadata.premine, &mut payload);

    if let Some(spacers) = etching.spacers {
        encode_varint(5, &mut payload); // "Spacers"
        encode_varint(spacers.into(), &mut payload);
    }

    if let Some(symbol) = etching.metadata.symbol.chars().next() {
        encode_varint(6, &mut payload); // "Symbol"
        encode_varint(symbol as u128, &mut payload);
    }

    // Encode Minting Rules
    if let Some(amount) = etching.amount {
        encode_varint(7, &mut payload); // "Mint Amount"
        encode_varint(amount, &mut payload);
    }

    encode_varint(8, &mut payload); // "Cap"
    encode_varint(etching.metadata.cap, &mut payload);

    if let Some((start, end)) = etching.height {
        encode_varint(9, &mut payload); // "Mint Height Range"
        encode_varint(start as u128, &mut payload);
        encode_varint(end as u128, &mut payload);
    }

    if let Some((start, end)) = etching.offset {
        encode_varint(10, &mut payload); // "Mint Offset Range"
        encode_varint(start as u128, &mut payload);
        encode_varint(end as u128, &mut payload);
    }

    // Finalize OP_RETURN Script
    let payload_pushbytes = PushBytesBuf::try_from(payload).map_err(|_| {
        BitcoinError::InternalError("Failed to encode Runestone payload".to_string())
    })?;

    Ok(Builder::new()
        .push_opcode(opcodes::all::OP_RETURN)
        .push_opcode(opcodes::all::OP_PUSHNUM_13) // Runestone Identifier
        .push_slice(&payload_pushbytes) // Encoded payload
        .into_script())
}

/// Encode a u128 integer as varint (following ordinals' style)
fn encode_varint(value: u128, payload: &mut Vec<u8>) {
    let mut temp = value;
    while temp >= 0x80 {
        payload.push((temp as u8) | 0x80);
        temp >>= 7;
    }
    payload.push(temp as u8);
}
