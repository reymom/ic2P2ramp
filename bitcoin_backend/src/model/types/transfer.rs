use candid::{CandidType, Deserialize};

use crate::RuneID;

#[derive(Deserialize, CandidType, Clone)]
pub enum TransactionType {
    LegacyBitcoin,          // Use p2pkh
    SimpleTaprootBitcoin,   // Use p2tr_raw_key_spend
    ScriptedTaprootBitcoin, // Use p2tr_script_spend
    RuneTransfer(RuneID),   // Use p2tr_script_spend with Rune script
}
