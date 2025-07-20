use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use std::collections::HashMap;

pub type Address = String; // we store Solana addresses in text form.

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct VaultEntry {
    pub lamports: u64,
    /// Mapping token‐mint → token‐balance (in smallest units)
    pub tokens: HashMap<String, u64>,
}

const MAX_VAULT_ENTRY_SIZE: u32 = 2048;

impl VaultEntry {
    pub fn new() -> Self {
        Self {
            lamports: 0,
            tokens: HashMap::new(),
        }
    }
}

impl Storable for VaultEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), VaultEntry).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VAULT_ENTRY_SIZE,
        is_fixed_size: false,
    };
}
