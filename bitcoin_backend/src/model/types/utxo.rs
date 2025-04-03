use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Hash, PartialEq, Eq, Clone, Debug)]
pub struct RuneUTXOEntry {
    pub txid: String,
    pub vout: u32,
    pub rune_amount: u64,
    pub script_pubkey: String,
}
