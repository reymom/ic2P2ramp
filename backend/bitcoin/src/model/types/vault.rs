use std::collections::HashMap;

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{Storable, storable::Bound};

use super::runes::RuneID;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VaultEntry {
    pub bitcoin_balance: u64,        // BTC balance in Satoshi
    pub runes: HashMap<RuneID, u64>, // Mapping of RuneID to balance
}

const MAX_VAULT_ENTRY_SIZE: u32 = 1024;

impl Storable for VaultEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VAULT_ENTRY_SIZE,
        is_fixed_size: false,
    };
}

impl VaultEntry {
    pub fn new() -> Self {
        Self {
            bitcoin_balance: 0,
            runes: HashMap::new(),
        }
    }
}
