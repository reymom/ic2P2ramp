use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RuneUTXOEntry {
    pub txid: String,
    pub vout: u32,
    pub rune_amount: u64,
    pub script_pubkey: String,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct RuneUTXOList(pub Vec<RuneUTXOEntry>);

impl Storable for RuneUTXOList {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        RuneUTXOList(Decode!(bytes.as_ref(), Vec<RuneUTXOEntry>).unwrap())
    }

    const BOUND: Bound = Bound::Unbounded;
}
